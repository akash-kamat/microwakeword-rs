use std::path::PathBuf;

use micro_wakeword::{AUDIO_BLOCK_SAMPLES, Detector, Runtime};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args_os().skip(1);
    let config = PathBuf::from(
        args.next()
            .ok_or("usage: custom_runtime CONFIG.json TENSORFLOW_LITE_LIBRARY")?,
    );
    let library = PathBuf::from(args.next().ok_or("missing TensorFlow Lite library path")?);

    let mut detector = Detector::from_config_with_runtime(config, Runtime::from_path(&library))?;
    detector.process_audio(&[0; AUDIO_BLOCK_SAMPLES])?;
    println!(
        "Loaded '{}' using {}",
        detector.config().wake_word,
        library.display()
    );
    Ok(())
}
