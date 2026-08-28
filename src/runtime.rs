use std::path::{Path, PathBuf};
use std::sync::Arc;

use tflite_c::TfLiteLibrary;

use crate::{Error, Result};

/// Selects the TensorFlow Lite C runtime used for inference.
#[derive(Clone, Debug, Default)]
pub enum Runtime {
    /// Check explicit environment overrides, then use the bundled runtime.
    #[default]
    Auto,
    /// Load a specific shared library.
    Path(PathBuf),
    /// Search standard system locations supported by `tflite-c-rs`.
    System,
}

impl Runtime {
    pub fn from_path(path: impl Into<PathBuf>) -> Self {
        Self::Path(path.into())
    }

    pub(crate) fn load(&self) -> Result<Arc<TfLiteLibrary>> {
        let explicit = match self {
            Self::Path(path) => Some(path.clone()),
            Self::Auto => std::env::var_os("MICRO_WAKEWORD_TFLITE_LIB")
                .or_else(|| std::env::var_os("TFLITE_C_LIB"))
                .map(PathBuf::from),
            Self::System => None,
        };
        if let Some(path) = explicit {
            return TfLiteLibrary::load_from_path(&path).map_err(Error::from);
        }
        match self {
            Self::Auto => {
                let path = bundled_runtime_path()?;
                TfLiteLibrary::load_from_path(path).map_err(Error::from)
            }
            Self::System => TfLiteLibrary::load_default().map_err(Error::from),
            Self::Path(_) => unreachable!(),
        }
    }

    pub fn path(&self) -> Option<&Path> {
        match self {
            Self::Path(path) => Some(path),
            Self::Auto | Self::System => None,
        }
    }
}

#[cfg(all(target_os = "windows", target_arch = "x86_64"))]
fn bundled_runtime_path() -> Result<PathBuf> {
    use std::fs;
    use std::io::Write;

    const BYTES: &[u8] = include_bytes!("../runtime/windows-x86_64/tensorflowlite_c-2.17.1.dll");
    const SHA256: &str = "882e6d8f9866ff84f23d4b964c145b7f0f0a8907fa830dcd8c499e7c46bf3365";
    let local = std::env::var_os("LOCALAPPDATA").ok_or_else(|| {
        Error::UnsupportedPlatform("LOCALAPPDATA is unavailable for runtime extraction".into())
    })?;
    let directory = PathBuf::from(local)
        .join("micro-wakeword")
        .join("runtime-2.17.1");
    let destination = directory.join("tensorflowlite_c.dll");
    if destination.exists() && file_sha256(&destination)? == SHA256 {
        return Ok(destination);
    }

    fs::create_dir_all(&directory).map_err(|source| Error::Io {
        path: directory.clone(),
        source,
    })?;
    let temporary = directory.join(format!("tensorflowlite_c.{}.tmp", std::process::id()));
    let mut file = fs::File::create(&temporary).map_err(|source| Error::Io {
        path: temporary.clone(),
        source,
    })?;
    file.write_all(BYTES)
        .and_then(|_| file.sync_all())
        .map_err(|source| Error::Io {
            path: temporary.clone(),
            source,
        })?;
    if file_sha256(&temporary)? != SHA256 {
        return Err(Error::InvalidConfig(
            "embedded TensorFlow Lite runtime checksum mismatch".into(),
        ));
    }
    if destination.exists() {
        fs::remove_file(&destination).map_err(|source| Error::Io {
            path: destination.clone(),
            source,
        })?;
    }
    if let Err(source) = fs::rename(&temporary, &destination) {
        if destination.exists() && file_sha256(&destination)? == SHA256 {
            let _ = fs::remove_file(&temporary);
        } else {
            return Err(Error::Io {
                path: destination,
                source,
            });
        }
    }
    Ok(destination)
}

#[cfg(all(target_os = "windows", target_arch = "x86_64"))]
fn file_sha256(path: &Path) -> Result<String> {
    use sha2::{Digest, Sha256};
    let bytes = std::fs::read(path).map_err(|source| Error::Io {
        path: path.to_owned(),
        source,
    })?;
    Ok(format!("{:x}", Sha256::digest(bytes)))
}

#[cfg(not(all(target_os = "windows", target_arch = "x86_64")))]
fn bundled_runtime_path() -> Result<PathBuf> {
    Err(Error::UnsupportedPlatform(
        "the bundled runtime currently supports only Windows x86-64; use Runtime::Path or Runtime::System".into(),
    ))
}
