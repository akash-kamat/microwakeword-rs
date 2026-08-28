use std::path::PathBuf;

use micro_wakeword::Listener;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args_os().skip(1);
    let model = PathBuf::from(args.next().ok_or(
        "usage: model_only MODEL.tflite WAKE_WORD PROBABILITY_CUTOFF SLIDING_WINDOW_SIZE",
    )?);
    let wake_word = args.next().ok_or("missing WAKE_WORD")?;
    let cutoff: f32 = args
        .next()
        .ok_or("missing PROBABILITY_CUTOFF")?
        .to_string_lossy()
        .parse()?;
    let window: usize = args
        .next()
        .ok_or("missing SLIDING_WINDOW_SIZE")?
        .to_string_lossy()
        .parse()?;

    let mut listener = Listener::builder(model)
        .wake_word(wake_word.to_string_lossy())
        .probability_cutoff(cutoff)
        .sliding_window_size(window)
        .build()?;

    println!(
        "Listening for {}. Press Ctrl+C to stop.",
        listener.detector().config().wake_word
    );
    while let Some(detection) = listener.next_detection()? {
        println!(
            "Detected {} ({:.1}%)",
            detection.wake_word,
            detection.probability * 100.0
        );
    }
    Ok(())
}
