use std::path::PathBuf;

use micro_wakeword::{Error, Listener};

fn main() {
    if let Err(error) = listen() {
        match error {
            Error::Io { path, source } => {
                eprintln!("Could not read {}: {source}", path.display());
            }
            Error::Json { path, source } => {
                eprintln!("Invalid JSON in {}: {source}", path.display());
            }
            Error::InvalidConfig(message) => eprintln!("Invalid settings: {message}"),
            Error::IncompatibleModel(message) => eprintln!("Incompatible model: {message}"),
            Error::Audio(message) if message.contains("queue overflowed") => {
                eprintln!("Audio was dropped because processing fell behind: {message}");
            }
            Error::Audio(message) => eprintln!("Microphone error: {message}"),
            Error::AudioStreamEnded => eprintln!("The microphone stream ended."),
            Error::UnsupportedPlatform(message) => eprintln!("Unsupported platform: {message}"),
            Error::Tflite(error) => eprintln!("TensorFlow Lite failed: {error}"),
        }
        std::process::exit(1);
    }
}

fn listen() -> micro_wakeword::Result<()> {
    let config = std::env::args_os()
        .nth(1)
        .map(PathBuf::from)
        .ok_or_else(|| Error::InvalidConfig("usage: error_handling CONFIG.json".into()))?;
    let mut listener = Listener::from_config(config)?;
    println!("Listening. Press Ctrl+C to stop.");
    while let Some(detection) = listener.next_detection()? {
        println!("Detected {}", detection.wake_word);
    }
    Ok(())
}
