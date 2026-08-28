use std::path::PathBuf;

/// Errors returned by this crate.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("could not read {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("invalid JSON in {path}: {source}")]
    Json {
        path: PathBuf,
        #[source]
        source: serde_json::Error,
    },
    #[error("invalid microWakeWord configuration: {0}")]
    InvalidConfig(String),
    #[error("incompatible microWakeWord model: {0}")]
    IncompatibleModel(String),
    #[error("TensorFlow Lite error: {0}")]
    Tflite(#[from] tflite_c::Error),
    #[error("unsupported platform: {0}")]
    UnsupportedPlatform(String),
    #[error("audio error: {0}")]
    Audio(String),
    #[error("the audio stream ended")]
    AudioStreamEnded,
}

pub type Result<T> = std::result::Result<T, Error>;
