use std::path::PathBuf;

use micro_wakeword::Listener;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = std::env::args_os()
        .nth(1)
        .map(PathBuf::from)
        .ok_or("usage: detector_access CONFIG.json")?;
    let mut listener = Listener::from_config(config)?;

    let settings = listener.detector().config();
    println!("Wake word: {}", settings.wake_word);
    println!("Model: {}", settings.model_path.display());
    println!("Cutoff: {}", settings.probability_cutoff);
    println!("Sliding window: {}", settings.sliding_window_size);

    // Mutable access is available for advanced operations such as resetting.
    listener.detector_mut().reset()?;
    println!("Detector reset; listening now. Press Ctrl+C to stop.");
    while let Some(detection) = listener.next_detection()? {
        println!("Detected {}", detection.wake_word);
    }
    Ok(())
}
