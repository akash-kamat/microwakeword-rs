use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use sha2::{Digest, Sha256};
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
    /// Selects a TensorFlow Lite C shared library at an explicit path.
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

    /// Returns the explicit library path for [`Runtime::Path`].
    pub fn path(&self) -> Option<&Path> {
        match self {
            Self::Path(path) => Some(path),
            Self::Auto | Self::System => None,
        }
    }
}

struct BundledRuntime {
    bytes: &'static [u8],
    sha256: &'static str,
    version: &'static str,
    target: &'static str,
    file_name: &'static str,
}

#[cfg(all(target_os = "windows", target_arch = "x86_64"))]
fn bundled_runtime() -> BundledRuntime {
    BundledRuntime {
        bytes: include_bytes!("../runtime/windows-x86_64/tensorflowlite_c-2.17.1.dll"),
        sha256: "882e6d8f9866ff84f23d4b964c145b7f0f0a8907fa830dcd8c499e7c46bf3365",
        version: "2.17.1",
        target: "x86_64-pc-windows",
        file_name: "tensorflowlite_c.dll",
    }
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
fn bundled_runtime() -> BundledRuntime {
    BundledRuntime {
        bytes: include_bytes!("../runtime/linux-x86_64/libtensorflowlite_c-2.17.1.so"),
        sha256: "25465edb5cd7aadd00249d4d28f1d922ce5cc90195ad4403458111dd63493bae",
        version: "2.17.1",
        target: "x86_64-unknown-linux-gnu",
        file_name: "libtensorflowlite_c.so",
    }
}

#[cfg(all(target_os = "linux", target_arch = "aarch64"))]
fn bundled_runtime() -> BundledRuntime {
    BundledRuntime {
        bytes: include_bytes!("../runtime/linux-aarch64/libtensorflowlite_c-2.17.1.so"),
        sha256: "12062437bfde367b1be592ebc4a9fe64b8453f90b3b7dc21fade002c38e1ac1a",
        version: "2.17.1",
        target: "aarch64-unknown-linux-gnu",
        file_name: "libtensorflowlite_c.so",
    }
}

#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
fn bundled_runtime() -> BundledRuntime {
    BundledRuntime {
        bytes: include_bytes!("../runtime/macos-aarch64/libtensorflowlite_c-2.17.1.dylib"),
        sha256: "e77597b3710e43f58f1c37a9c8979a82901ed9f3a11de71e95f2154a4c9ce6d7",
        version: "2.17.1",
        target: "aarch64-apple-darwin",
        file_name: "libtensorflowlite_c.dylib",
    }
}

#[cfg(all(target_os = "macos", target_arch = "x86_64"))]
fn bundled_runtime() -> BundledRuntime {
    BundledRuntime {
        bytes: include_bytes!("../runtime/macos-x86_64/libtensorflowlite_c-2.17.0.dylib"),
        sha256: "6cf562771e6cb8d7856a86f859d004f0ef72861b6cafa88afbfad5d5f40261fc",
        version: "2.17.0",
        target: "x86_64-apple-darwin",
        file_name: "libtensorflowlite_c.dylib",
    }
}

#[cfg(any(
    all(target_os = "windows", target_arch = "x86_64"),
    all(
        target_os = "linux",
        any(target_arch = "x86_64", target_arch = "aarch64")
    ),
    all(
        target_os = "macos",
        any(target_arch = "x86_64", target_arch = "aarch64")
    ),
))]
fn bundled_runtime_path() -> Result<PathBuf> {
    let runtime = bundled_runtime();
    let directory = runtime_cache_root()
        .join("micro-wakeword")
        .join(format!("runtime-{}-{}", runtime.version, runtime.target));
    let destination = directory.join(runtime.file_name);
    if has_expected_checksum(&destination, runtime.sha256)? {
        return Ok(destination);
    }

    fs::create_dir_all(&directory).map_err(|source| Error::Io {
        path: directory.clone(),
        source,
    })?;

    static TEMPORARY_ID: AtomicU64 = AtomicU64::new(0);
    let id = TEMPORARY_ID.fetch_add(1, Ordering::Relaxed);
    let temporary = directory.join(format!(
        ".{}.{}.{id}.tmp",
        runtime.file_name,
        std::process::id()
    ));
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temporary)
        .map_err(|source| Error::Io {
            path: temporary.clone(),
            source,
        })?;
    if let Err(source) = file.write_all(runtime.bytes).and_then(|_| file.sync_all()) {
        let _ = fs::remove_file(&temporary);
        return Err(Error::Io {
            path: temporary,
            source,
        });
    }
    drop(file);

    if !has_expected_checksum(&temporary, runtime.sha256)? {
        let _ = fs::remove_file(&temporary);
        return Err(Error::InvalidConfig(
            "embedded TensorFlow Lite runtime checksum mismatch".into(),
        ));
    }

    // Another process may have completed extraction while this process wrote
    // its temporary file. Reuse that verified file instead of replacing a
    // library which may already be mapped by the other process.
    if has_expected_checksum(&destination, runtime.sha256)? {
        let _ = fs::remove_file(&temporary);
        return Ok(destination);
    }
    if destination.exists() {
        fs::remove_file(&destination).map_err(|source| Error::Io {
            path: destination.clone(),
            source,
        })?;
    }
    if let Err(source) = fs::rename(&temporary, &destination) {
        if has_expected_checksum(&destination, runtime.sha256)? {
            let _ = fs::remove_file(&temporary);
        } else {
            let _ = fs::remove_file(&temporary);
            return Err(Error::Io {
                path: destination,
                source,
            });
        }
    }
    Ok(destination)
}

#[cfg(any(
    all(target_os = "windows", target_arch = "x86_64"),
    all(
        target_os = "linux",
        any(target_arch = "x86_64", target_arch = "aarch64")
    ),
    all(
        target_os = "macos",
        any(target_arch = "x86_64", target_arch = "aarch64")
    ),
))]
fn runtime_cache_root() -> PathBuf {
    #[cfg(target_os = "windows")]
    if let Some(path) = std::env::var_os("LOCALAPPDATA") {
        return PathBuf::from(path);
    }

    #[cfg(target_os = "linux")]
    if let Some(path) = std::env::var_os("XDG_CACHE_HOME") {
        return PathBuf::from(path);
    }

    #[cfg(target_os = "macos")]
    if let Some(path) = std::env::var_os("HOME") {
        return PathBuf::from(path).join("Library").join("Caches");
    }

    #[cfg(target_os = "linux")]
    if let Some(path) = std::env::var_os("HOME") {
        return PathBuf::from(path).join(".cache");
    }

    std::env::temp_dir()
}

#[cfg(any(
    all(target_os = "windows", target_arch = "x86_64"),
    all(
        target_os = "linux",
        any(target_arch = "x86_64", target_arch = "aarch64")
    ),
    all(
        target_os = "macos",
        any(target_arch = "x86_64", target_arch = "aarch64")
    ),
))]
fn has_expected_checksum(path: &Path, expected: &str) -> Result<bool> {
    if !path.exists() {
        return Ok(false);
    }
    Ok(file_sha256(path)? == expected)
}

#[cfg(any(
    all(target_os = "windows", target_arch = "x86_64"),
    all(
        target_os = "linux",
        any(target_arch = "x86_64", target_arch = "aarch64")
    ),
    all(
        target_os = "macos",
        any(target_arch = "x86_64", target_arch = "aarch64")
    ),
))]
fn file_sha256(path: &Path) -> Result<String> {
    let file = fs::File::open(path).map_err(|source| Error::Io {
        path: path.to_owned(),
        source,
    })?;
    let mut reader = std::io::BufReader::new(file);
    let mut digest = Sha256::new();
    let mut buffer = [0_u8; 64 * 1024];
    loop {
        let read = reader.read(&mut buffer).map_err(|source| Error::Io {
            path: path.to_owned(),
            source,
        })?;
        if read == 0 {
            break;
        }
        digest.update(&buffer[..read]);
    }
    Ok(format!("{:x}", digest.finalize()))
}

#[cfg(not(any(
    all(target_os = "windows", target_arch = "x86_64"),
    all(
        target_os = "linux",
        any(target_arch = "x86_64", target_arch = "aarch64")
    ),
    all(
        target_os = "macos",
        any(target_arch = "x86_64", target_arch = "aarch64")
    ),
)))]
fn bundled_runtime_path() -> Result<PathBuf> {
    Err(Error::UnsupportedPlatform(
        "no bundled TensorFlow Lite runtime for this target; use Runtime::Path or Runtime::System"
            .into(),
    ))
}

#[cfg(test)]
mod tests {
    #[cfg(any(
        all(target_os = "windows", target_arch = "x86_64"),
        all(
            target_os = "linux",
            any(target_arch = "x86_64", target_arch = "aarch64")
        ),
        all(
            target_os = "macos",
            any(target_arch = "x86_64", target_arch = "aarch64")
        ),
    ))]
    #[test]
    fn bundled_runtime_loads_all_required_symbols() {
        let path = super::bundled_runtime_path().unwrap();
        tflite_c::TfLiteLibrary::load_from_path(path).unwrap();
    }
}
