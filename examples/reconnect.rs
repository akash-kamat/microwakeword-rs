use std::path::PathBuf;
use std::thread;
use std::time::Duration;

use micro_wakeword::{Error, Listener};

const RECONNECT_DELAY: Duration = Duration::from_secs(2);

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = std::env::args_os()
        .nth(1)
        .map(PathBuf::from)
        .ok_or("usage: reconnect CONFIG.json")?;

    loop {
        let mut listener = match Listener::from_config(&config) {
            Ok(listener) => listener,
            Err(error) if is_microphone_error(&error) => {
                eprintln!("Microphone unavailable ({error}); retrying...");
                thread::sleep(RECONNECT_DELAY);
                continue;
            }
            Err(error) => return Err(error.into()),
        };

        println!(
            "Listening for {} on the current default microphone.",
            listener.detector().config().wake_word
        );

        loop {
            match listener.next_detection() {
                Ok(Some(detection)) => println!(
                    "Detected {} ({:.1}%)",
                    detection.wake_word,
                    detection.probability * 100.0
                ),
                Ok(None) => return Ok(()),
                Err(error) if is_microphone_error(&error) => {
                    eprintln!("Microphone disconnected ({error}); retrying...");
                    break;
                }
                Err(error) => return Err(error.into()),
            }
        }

        // Recreating a default-device Listener asks the OS for its current
        // default input; the library never chooses an arbitrary fallback.
        drop(listener);
        thread::sleep(RECONNECT_DELAY);
    }
}

fn is_microphone_error(error: &Error) -> bool {
    matches!(error, Error::Audio(_) | Error::AudioStreamEnded)
}
