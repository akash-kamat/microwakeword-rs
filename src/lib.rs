//! Detection and live microphone listening for microWakeWord models.

mod config;
mod detector;
mod error;
mod features;
mod runtime;

#[cfg(feature = "listener")]
mod audio;
#[cfg(feature = "listener")]
mod listener;

pub use config::{Config, ModelMetadata};
pub use detector::{Detection, Detector, DetectorBuilder};
pub use error::{Error, Result};
pub use runtime::Runtime;

#[cfg(feature = "listener")]
pub use audio::{AudioDevice, available_input_devices};
#[cfg(feature = "listener")]
pub use listener::{DEFAULT_COOLDOWN, Listener, ListenerBuilder};

/// Required input sample rate.
pub const SAMPLE_RATE: u32 = 16_000;
/// Samples in one 10 ms mono PCM block.
pub const AUDIO_BLOCK_SAMPLES: usize = 160;
