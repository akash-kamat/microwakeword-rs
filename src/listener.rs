use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, mpsc};
use std::time::{Duration, Instant};

use cpal::Stream;
use cpal::traits::StreamTrait;

use crate::audio::{AudioEvent, open_input};
use crate::{Config, Detection, Detector, Error, Result, Runtime};

/// The repeat-suppression period used by [`Listener`] unless overridden.
pub const DEFAULT_COOLDOWN: Duration = Duration::from_secs(1);

/// A live microphone stream connected to a detector.
pub struct Listener {
    detector: Detector,
    receiver: mpsc::Receiver<AudioEvent>,
    dropped: Arc<AtomicUsize>,
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
        let dropped = Arc::new(AtomicUsize::new(0));
        let stream = open_input(device, sender, dropped.clone())?;
        stream.play().map_err(|e| Error::Audio(e.to_string()))?;
        Ok(Self {
            detector,
            receiver,
            dropped,
            cooldown: Cooldown::new(cooldown),
            _stream: stream,
        })
    }

    /// Block until a detection occurs or the audio stream reports an error.
    pub fn next_detection(&mut self) -> Result<Option<Detection>> {
        loop {
            let dropped = self.dropped.swap(0, Ordering::Relaxed);
            if dropped != 0 {
                self.detector.reset()?;
                return Err(Error::Audio(format!(
                    "microphone queue overflowed; dropped {dropped} audio blocks"
                )));
            }
            match self.receiver.recv() {
                Ok(AudioEvent::Samples(samples)) => {
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
                Ok(AudioEvent::Error(error)) => return Err(Error::Audio(error)),
                Err(_) => return Err(Error::AudioStreamEnded),
            }
        }
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
            if let Some(config) = self.config {
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

#[cfg(test)]
mod tests {
    use super::*;

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
}
