use std::fs;
use std::path::{Path, PathBuf};

use micro_wakeword::{AUDIO_BLOCK_SAMPLES, Detector, SAMPLE_RATE};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args_os().skip(1);
    let config = PathBuf::from(
        args.next()
            .ok_or("usage: detect_pcm CONFIG.json [AUDIO.raw]")?,
    );
    let samples = match args.next() {
        Some(path) => read_raw_pcm(Path::new(&path))?,
        None => {
            println!("No audio file supplied; processing three seconds of silence.");
            vec![0; SAMPLE_RATE as usize * 3]
        }
    };

    let mut detector = Detector::from_config(config)?;
    for block in samples.chunks(AUDIO_BLOCK_SAMPLES) {
        if block.len() != AUDIO_BLOCK_SAMPLES {
            return Err(format!(
                "audio ends with an incomplete block of {} samples",
                block.len()
            )
            .into());
        }
        if let Some(detection) = detector.process_audio(block)? {
            // Detector intentionally has no cooldown. The caller decides how
            // to handle repeated detections.
            println!(
                "Detected {} ({:.1}%)",
                detection.wake_word,
                detection.probability * 100.0
            );
        }
    }

    // Reset before an unrelated stream or after an audio discontinuity.
    detector.reset()?;
    println!("Finished processing audio and reset the detector.");
    Ok(())
}

fn read_raw_pcm(path: &Path) -> Result<Vec<i16>, Box<dyn std::error::Error>> {
    let bytes = fs::read(path)?;
    if bytes.len() % 2 != 0 {
        return Err("raw PCM contains an incomplete i16 sample".into());
    }
    Ok(bytes
        .chunks(2)
        .map(|pair| i16::from_le_bytes([pair[0], pair[1]]))
        .collect())
}
