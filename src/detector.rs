use std::collections::VecDeque;
use std::path::{Path, PathBuf};

use tflite_c::{Interpreter, InterpreterOptions, Model, TfLiteType};

use crate::config::{Config, ModelMetadata};
use crate::features::{FEATURE_COUNT, FEATURE_SCALE, Frontend};
use crate::{AUDIO_BLOCK_SAMPLES, Error, Result, Runtime};

/// A thresholded wake-word prediction.
#[derive(Clone, Debug, PartialEq)]
pub struct Detection {
    pub wake_word: String,
    /// Averaged probability in the inclusive range 0–1.
    pub probability: f32,
}

/// Streaming detector for 16 kHz mono signed 16-bit PCM.
pub struct Detector {
    config: Config,
    runtime: Runtime,
    frontend: Frontend,
    engine: Engine,
    feature_rows: Vec<[u16; FEATURE_COUNT]>,
    probabilities: VecDeque<f32>,
}

struct Engine {
    interpreter: Interpreter,
    input_scale: f32,
    input_zero_point: i32,
    model_rows: usize,
}

impl Engine {
    fn new(config: &Config, runtime: &Runtime) -> Result<Self> {
        let library = runtime.load()?;
        let model = Model::from_file(&config.model_path, library.clone())?;
        let mut options = InterpreterOptions::new(library);
        options.num_threads(1);
        let interpreter = Interpreter::new(model, options)?;

        if interpreter.input_count() != 1 {
            return Err(Error::IncompatibleModel(format!(
                "expected one input tensor, got {}",
                interpreter.input_count()
            )));
        }
        if interpreter.output_count() != 1 {
            return Err(Error::IncompatibleModel(format!(
                "expected one output tensor, got {}",
                interpreter.output_count()
            )));
        }
        let input = interpreter.input(0)?;
        let dims = input.dims();
        if dims.len() != 3 || dims[0] != 1 || dims[1] <= 0 || dims[2] != FEATURE_COUNT as i32 {
            return Err(Error::IncompatibleModel(format!(
                "expected int8 input [1, rows, {FEATURE_COUNT}], got {:?}",
                dims
            )));
        }
        if input.dtype() != TfLiteType::Int8 {
            return Err(Error::IncompatibleModel(format!(
                "expected int8 input, got {:?}",
                input.dtype()
            )));
        }
        let quantization = input.quantization();
        if !quantization.scale.is_finite() || quantization.scale <= 0.0 {
            return Err(Error::IncompatibleModel(
                "input tensor has invalid quantization scale".into(),
            ));
        }
        let model_rows = dims[1] as usize;
        let output = interpreter.output(0)?;
        let output_elements = output
            .dims()
            .iter()
            .try_fold(1_i32, |n, dim| n.checked_mul(*dim));
        if output_elements != Some(1) {
            return Err(Error::IncompatibleModel(format!(
                "expected a single probability output, got shape {:?}",
                output.dims()
            )));
        }
        if !matches!(
            output.dtype(),
            TfLiteType::Int8 | TfLiteType::UInt8 | TfLiteType::Float32
        ) {
            return Err(Error::IncompatibleModel(format!(
                "expected a quantized or float probability output, got {:?}",
                output.dtype()
            )));
        }
        if output.dtype() != TfLiteType::Float32 {
            let quantization = output.quantization();
            if !quantization.scale.is_finite() || quantization.scale <= 0.0 {
                return Err(Error::IncompatibleModel(
                    "output tensor has invalid quantization scale".into(),
                ));
            }
        }

        Ok(Self {
            interpreter,
            input_scale: quantization.scale,
            input_zero_point: quantization.zero_point,
            model_rows,
        })
    }
}

impl Detector {
    pub fn from_config(path: impl AsRef<Path>) -> Result<Self> {
        Self::from_config_with_runtime(path, Runtime::Auto)
    }

    pub fn from_config_with_runtime(path: impl AsRef<Path>, runtime: Runtime) -> Result<Self> {
        Self::from_parts(Config::from_file(path)?, runtime)
    }

    pub fn builder(model_path: impl Into<PathBuf>) -> DetectorBuilder {
        DetectorBuilder::new(model_path)
    }

    pub(crate) fn from_parts(config: Config, runtime: Runtime) -> Result<Self> {
        config.validate()?;
        let frontend = Frontend::new()?;
        let engine = Engine::new(&config, &runtime)?;
        let model_rows = engine.model_rows;
        let window = config.sliding_window_size;
        Ok(Self {
            config,
            runtime,
            frontend,
            engine,
            feature_rows: Vec::with_capacity(model_rows),
            probabilities: VecDeque::with_capacity(window),
        })
    }

    pub fn config(&self) -> &Config {
        &self.config
    }

    /// Consume exactly one 10 ms audio block (160 samples).
    pub fn process_audio(&mut self, samples: &[i16]) -> Result<Option<Detection>> {
        let samples: &[i16; AUDIO_BLOCK_SAMPLES] = samples.try_into().map_err(|_| {
            Error::Audio(format!(
                "expected exactly {AUDIO_BLOCK_SAMPLES} samples of 16 kHz mono PCM, got {}",
                samples.len()
            ))
        })?;
        let Some(features) = self.frontend.process(samples)? else {
            return Ok(None);
        };
        self.process_features(features)
    }

    fn process_features(&mut self, features: [u16; FEATURE_COUNT]) -> Result<Option<Detection>> {
        self.feature_rows.push(features);
        if self.feature_rows.len() < self.engine.model_rows {
            return Ok(None);
        }

        {
            let mut input = self.engine.interpreter.input_mut(0)?;
            let destination = input.data_mut()?;
            for (target, feature) in destination
                .iter_mut()
                .zip(self.feature_rows.iter().flat_map(|row| row.iter().copied()))
            {
                let real = feature as f32 * FEATURE_SCALE;
                let quantized = (real / self.engine.input_scale
                    + self.engine.input_zero_point as f32)
                    .round()
                    .clamp(i8::MIN as f32, i8::MAX as f32) as i8;
                *target = quantized as u8;
            }
        }
        self.feature_rows.clear();
        self.engine.interpreter.invoke()?;
        let output = self.engine.interpreter.output(0)?.to_vec_f32()?;
        let probability = output[0].clamp(0.0, 1.0);
        self.probabilities.push_back(probability);
        if self.probabilities.len() > self.config.sliding_window_size {
            self.probabilities.pop_front();
        }
        if self.probabilities.len() < self.config.sliding_window_size {
            return Ok(None);
        }
        let probability = self.probabilities.iter().sum::<f32>() / self.probabilities.len() as f32;
        Ok(
            passes_cutoff(probability, self.config.probability_cutoff).then(|| Detection {
                wake_word: self.config.wake_word.clone(),
                probability,
            }),
        )
    }

    /// Clear frontend history, probability history, and all model recurrent state.
    pub fn reset(&mut self) -> Result<()> {
        let frontend = Frontend::new()?;
        let engine = Engine::new(&self.config, &self.runtime)?;
        self.frontend = frontend;
        self.engine = engine;
        self.feature_rows.clear();
        self.probabilities.clear();
        Ok(())
    }
}

fn passes_cutoff(probability: f32, cutoff: f32) -> bool {
    probability > cutoff
}

/// Builder for detectors that do not use a model JSON file.
pub struct DetectorBuilder {
    model_path: PathBuf,
    wake_word: Option<String>,
    probability_cutoff: Option<f32>,
    sliding_window_size: Option<usize>,
    feature_step_size_ms: u32,
    runtime: Runtime,
}

impl DetectorBuilder {
    fn new(model_path: impl Into<PathBuf>) -> Self {
        Self {
            model_path: model_path.into(),
            wake_word: None,
            probability_cutoff: None,
            sliding_window_size: None,
            feature_step_size_ms: 10,
            runtime: Runtime::Auto,
        }
    }

    pub fn wake_word(mut self, value: impl Into<String>) -> Self {
        self.wake_word = Some(value.into());
        self
    }
    pub fn probability_cutoff(mut self, value: f32) -> Self {
        self.probability_cutoff = Some(value);
        self
    }
    pub fn sliding_window_size(mut self, value: usize) -> Self {
        self.sliding_window_size = Some(value);
        self
    }
    pub fn feature_step_size_ms(mut self, value: u32) -> Self {
        self.feature_step_size_ms = value;
        self
    }
    pub fn runtime(mut self, value: Runtime) -> Self {
        self.runtime = value;
        self
    }

    pub fn build(self) -> Result<Detector> {
        let missing = |name| Error::InvalidConfig(format!("builder requires `{name}`"));
        let config = Config {
            model_path: self.model_path,
            wake_word: self.wake_word.ok_or_else(|| missing("wake_word"))?,
            probability_cutoff: self
                .probability_cutoff
                .ok_or_else(|| missing("probability_cutoff"))?,
            sliding_window_size: self
                .sliding_window_size
                .ok_or_else(|| missing("sliding_window_size"))?,
            feature_step_size_ms: self.feature_step_size_ms,
            metadata: ModelMetadata {
                format_version: 2,
                ..ModelMetadata::default()
            },
        };
        Detector::from_parts(config, self.runtime)
    }
}

#[cfg(test)]
mod tests {
    use super::passes_cutoff;

    #[test]
    fn cutoff_boundary_is_strict_like_micro_wakeword() {
        assert!(!passes_cutoff(0.3, 0.3));
        assert!(passes_cutoff(0.300_001, 0.3));
    }
}
