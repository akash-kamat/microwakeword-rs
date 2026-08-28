use std::fs;
use std::path::{Path, PathBuf};

use serde::Deserialize;

use crate::{Error, Result};

/// Descriptive fields from a microWakeWord model file.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ModelMetadata {
    pub author: Option<String>,
    pub website: Option<String>,
    pub trained_languages: Vec<String>,
    pub format_version: u32,
}

/// Validated model and detection configuration.
#[derive(Clone, Debug)]
pub struct Config {
    pub model_path: PathBuf,
    pub wake_word: String,
    pub probability_cutoff: f32,
    pub sliding_window_size: usize,
    pub feature_step_size_ms: u32,
    pub metadata: ModelMetadata,
}

#[derive(Deserialize)]
struct FileConfig {
    #[serde(rename = "type")]
    kind: String,
    wake_word: String,
    model: PathBuf,
    version: u32,
    author: Option<String>,
    website: Option<String>,
    #[serde(default)]
    trained_languages: Vec<String>,
    micro: MicroConfig,
}

#[derive(Deserialize)]
struct MicroConfig {
    probability_cutoff: f32,
    sliding_window_size: usize,
    feature_step_size: u32,
}

impl Config {
    /// Parse a standard microWakeWord JSON file and resolve its model path
    /// relative to the JSON file.
    pub fn from_file(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        let text = fs::read_to_string(path).map_err(|source| Error::Io {
            path: path.to_owned(),
            source,
        })?;
        let raw: FileConfig = serde_json::from_str(&text).map_err(|source| Error::Json {
            path: path.to_owned(),
            source,
        })?;
        let parent = path.parent().unwrap_or_else(|| Path::new("."));
        let model_path = if raw.model.is_absolute() {
            raw.model
        } else {
            parent.join(raw.model)
        };
        let config = Self {
            model_path,
            wake_word: raw.wake_word,
            probability_cutoff: raw.micro.probability_cutoff,
            sliding_window_size: raw.micro.sliding_window_size,
            feature_step_size_ms: raw.micro.feature_step_size,
            metadata: ModelMetadata {
                author: raw.author,
                website: raw.website,
                trained_languages: raw.trained_languages,
                format_version: raw.version,
            },
        };
        config.validate_with_kind(&raw.kind)?;
        Ok(config)
    }

    pub(crate) fn validate(&self) -> Result<()> {
        self.validate_with_kind("micro")
    }

    fn validate_with_kind(&self, kind: &str) -> Result<()> {
        if kind != "micro" {
            return Err(Error::InvalidConfig(format!(
                "type must be `micro`, got `{kind}`"
            )));
        }
        if self.metadata.format_version != 2 {
            return Err(Error::InvalidConfig(format!(
                "only format version 2 is supported, got {}",
                self.metadata.format_version
            )));
        }
        if self.wake_word.trim().is_empty() {
            return Err(Error::InvalidConfig("wake_word cannot be empty".into()));
        }
        if !(0.0..=1.0).contains(&self.probability_cutoff) || !self.probability_cutoff.is_finite() {
            return Err(Error::InvalidConfig(
                "probability_cutoff must be between 0 and 1".into(),
            ));
        }
        if self.sliding_window_size == 0 {
            return Err(Error::InvalidConfig(
                "sliding_window_size must be greater than zero".into(),
            ));
        }
        if self.feature_step_size_ms != 10 {
            return Err(Error::InvalidConfig(format!(
                "only a 10 ms feature_step_size is supported, got {}",
                self.feature_step_size_ms
            )));
        }
        Ok(())
    }
}
