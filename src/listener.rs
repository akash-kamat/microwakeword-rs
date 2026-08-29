use std::path::{Path, PathBuf};
use std::sync::{Arc, mpsc};
use std::time::{Duration, Instant};

use cpal::Stream;
use cpal::traits::StreamTrait;

use crate::audio::{AudioStatus, open_input};
use crate::{Config, Detection, Detector, Error, Result, Runtime};

/// The repeat-suppression period used by [`Listener`] unless overridden.
pub const DEFAULT_COOLDOWN: Duration = Duration::from_secs(1);

/// A live microphone stream connected to a detector.
pub struct Listener {
    detector: Detector,
    receiver: mpsc::Receiver<[i16; crate::AUDIO_BLOCK_SAMPLES]>,
    audio_status: Arc<AudioStatus>,
    dropped_audio_blocks: usize,
    cooldown: Cooldown,
    _stream: Stream,
}

impl Listener {
    pub fn from_config(path: impl AsRef<Path>) -> Result<Self> {
        ListenerBuilder::from_config(path)?.build()
    }

    pub fn builder(model_path: impl Into<PathBuf>) -> ListenerBuilder {
        ListenerBuilder::new(model_path)
    }

    /// Configure runtime and microphone selection while loading model settings from JSON.
    pub fn config_builder(path: impl AsRef<Path>) -> Result<ListenerBuilder> {
        ListenerBuilder::from_config(path)
    }

    pub fn from_detector(detector: Detector, device: Option<&str>) -> Result<Self> {
        Self::from_detector_with_cooldown(detector, device, DEFAULT_COOLDOWN)
    }

    fn from_detector_with_cooldown(
        detector: Detector,
        device: Option<&str>,
        cooldown: Duration,
    ) -> Result<Self> {
        let (sender, receiver) = mpsc::sync_channel(32);
        let audio_status = Arc::new(AudioStatus::default());
        let stream = open_input(device, sender, audio_status.clone())?;
        stream.play().map_err(|e| Error::Audio(e.to_string()))?;
        Ok(Self {
            detector,
            receiver,
            audio_status,
            dropped_audio_blocks: 0,
            cooldown: Cooldown::new(cooldown),
            _stream: stream,
        })
    }

    /// Block until a detection occurs or the audio stream reports an error.
    pub fn next_detection(&mut self) -> Result<Option<Detection>> {
        loop {
            if let Some(error) = self.audio_status.take_error() {
                return Err(Error::Audio(error));
            }
            if self.recover_from_overflow()? {
                continue;
            }
            match self.receiver.recv_timeout(Duration::from_millis(50)) {
                Ok(samples) => {
                    // Recheck after receiving so a drop racing with the receive
                    // cannot feed discontinuous audio into the detector.
                    if self.recover_from_overflow()? {
                        continue;
                    }
                    if let Some(detection) = self.detector.process_audio(&samples)? {
                        let now = Instant::now();
                        if self.cooldown.accept(now) {
                            // Start the next accepted utterance with clean frontend,
                            // model, and probability-window state.
                            self.detector.reset()?;
                            return Ok(Some(detection));
                        }
                    }
                }
                Err(mpsc::RecvTimeoutError::Timeout) => continue,
                Err(mpsc::RecvTimeoutError::Disconnected) => return Err(Error::AudioStreamEnded),
            }
        }
    }

    fn recover_from_overflow(&mut self) -> Result<bool> {
        let dropped = self.audio_status.dropped();
        if dropped == 0 {
            return Ok(false);
        }

        while self.receiver.try_recv().is_ok() {}
        self.detector.reset()?;
        self.dropped_audio_blocks = self.dropped_audio_blocks.saturating_add(dropped);
        Ok(true)
    }

    pub fn detector(&self) -> &Detector {
        &self.detector
    }
    pub fn detector_mut(&mut self) -> &mut Detector {
        &mut self.detector
    }

    /// Duration for which repeated detections are suppressed.
    pub fn cooldown(&self) -> Duration {
        self.cooldown.duration
    }

    /// Number of audio blocks dropped and automatically recovered since this
    /// listener was created. Each block represents 10 milliseconds of audio.
    pub fn dropped_audio_blocks(&self) -> usize {
        self.dropped_audio_blocks
    }
}

struct Cooldown {
    duration: Duration,
    last_detection: Option<Instant>,
}

impl Cooldown {
    fn new(duration: Duration) -> Self {
        Self {
            duration,
            last_detection: None,
        }
    }

    fn accept(&mut self, now: Instant) -> bool {
        if self
            .last_detection
            .is_some_and(|last| now.saturating_duration_since(last) < self.duration)
        {
            return false;
        }
        self.last_detection = Some(now);
        true
    }
}

pub struct ListenerBuilder {
    config: Option<Config>,
    model_path: Option<PathBuf>,
    wake_word: Option<String>,
    probability_cutoff: Option<f32>,
    sliding_window_size: Option<usize>,
    runtime: Runtime,
    device: Option<String>,
    cooldown: Duration,
}

impl ListenerBuilder {
    fn new(model_path: impl Into<PathBuf>) -> Self {
        Self {
            config: None,
            model_path: Some(model_path.into()),
            wake_word: None,
            probability_cutoff: None,
            sliding_window_size: None,
            runtime: Runtime::Auto,
            device: None,
            cooldown: DEFAULT_COOLDOWN,
        }
    }

    pub fn from_config(path: impl AsRef<Path>) -> Result<Self> {
        Ok(Self {
            config: Some(Config::from_file(path)?),
            model_path: None,
            wake_word: None,
            probability_cutoff: None,
            sliding_window_size: None,
            runtime: Runtime::Auto,
            device: None,
            cooldown: DEFAULT_COOLDOWN,
        })
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
    pub fn runtime(mut self, value: Runtime) -> Self {
        self.runtime = value;
        self
    }
    pub fn device(mut self, name_or_index: impl Into<String>) -> Self {
        self.device = Some(name_or_index.into());
        self
    }

    /// Suppress repeated detections for this long after an accepted detection.
    ///
    /// Pass [`Duration::ZERO`] to report every detection produced by the
    /// underlying detector.
    pub fn cooldown(mut self, value: Duration) -> Self {
        self.cooldown = value;
        self
    }

    pub fn build(self) -> Result<Listener> {
        let detector =
            if let Some(mut config) = self.config {
                apply_config_overrides(
                    &mut config,
                    self.wake_word,
                    self.probability_cutoff,
                    self.sliding_window_size,
                );
                Detector::from_parts(config, self.runtime)?
            } else {
                Detector::builder(self.model_path.expect("builder model path is present"))
                    .wake_word(self.wake_word.ok_or_else(|| {
                        Error::InvalidConfig("builder requires `wake_word`".into())
                    })?)
                    .probability_cutoff(self.probability_cutoff.ok_or_else(|| {
                        Error::InvalidConfig("builder requires `probability_cutoff`".into())
                    })?)
                    .sliding_window_size(self.sliding_window_size.ok_or_else(|| {
                        Error::InvalidConfig("builder requires `sliding_window_size`".into())
                    })?)
                    .runtime(self.runtime)
                    .build()?
            };
        Listener::from_detector_with_cooldown(detector, self.device.as_deref(), self.cooldown)
    }
}

fn apply_config_overrides(
    config: &mut Config,
    wake_word: Option<String>,
    probability_cutoff: Option<f32>,
    sliding_window_size: Option<usize>,
) {
    if let Some(wake_word) = wake_word {
        config.wake_word = wake_word;
    }
    if let Some(probability_cutoff) = probability_cutoff {
        config.probability_cutoff = probability_cutoff;
    }
    if let Some(sliding_window_size) = sliding_window_size {
        config.sliding_window_size = sliding_window_size;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ModelMetadata;

    #[test]
    fn cooldown_suppresses_until_duration_has_elapsed() {
        let start = Instant::now();
        let mut cooldown = Cooldown::new(Duration::from_secs(2));

        assert!(cooldown.accept(start));
        assert!(!cooldown.accept(start + Duration::from_millis(1_999)));
        assert!(cooldown.accept(start + Duration::from_secs(2)));
    }

    #[test]
    fn default_cooldown_is_one_second() {
        assert_eq!(DEFAULT_COOLDOWN, Duration::from_secs(1));
    }

    #[test]
    fn zero_cooldown_accepts_every_detection() {
        let now = Instant::now();
        let mut cooldown = Cooldown::new(Duration::ZERO);

        assert!(cooldown.accept(now));
        assert!(cooldown.accept(now));
    }

    #[test]
    fn config_builder_overrides_are_applied() {
        let mut config = Config {
            model_path: "model.tflite".into(),
            wake_word: "original".into(),
            probability_cutoff: 0.5,
            sliding_window_size: 3,
            feature_step_size_ms: 10,
            metadata: ModelMetadata {
                author: None,
                website: None,
                trained_languages: Vec::new(),
                format_version: 2,
            },
        };

        apply_config_overrides(&mut config, Some("override".into()), Some(0.75), Some(5));

        assert_eq!(config.wake_word, "override");
        assert_eq!(config.probability_cutoff, 0.75);
        assert_eq!(config.sliding_window_size, 5);
    }
}
