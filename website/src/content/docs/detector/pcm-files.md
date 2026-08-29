---
title: Process PCM files
description: Run the detector over headerless 16 kHz mono i16 audio.
---

The repository's `detect_pcm` example reads headerless little-endian PCM. This format contains samples only—no WAV header or metadata.

```bash
cargo run --no-default-features --example detect_pcm -- wake-word.json speech.raw
```

## Core implementation

```rust
use std::fs;
use micro_wakeword::{AUDIO_BLOCK_SAMPLES, Detector};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let bytes = fs::read("speech.raw")?;
    if bytes.len() % 2 != 0 {
        return Err("incomplete i16 sample".into());
    }

    let samples: Vec<i16> = bytes
        .chunks_exact(2)
        .map(|pair| i16::from_le_bytes([pair[0], pair[1]]))
        .collect();

    let mut detector = Detector::from_config("wake-word.json")?;
    for block in samples.chunks_exact(AUDIO_BLOCK_SAMPLES) {
        if let Some(hit) = detector.process_audio(block)? {
            println!("{}: {:.1}%", hit.wake_word, hit.probability * 100.0);
        }
    }
    Ok(())
}
```

## Convert a file with FFmpeg

Convert almost any decodable audio file into the exact expected representation:

```bash
ffmpeg -i input.wav -ac 1 -ar 16000 -f s16le speech.raw
```

`-ac 1` selects mono, `-ar 16000` resamples to 16 kHz, and `-f s16le` writes signed 16-bit little-endian PCM.

:::caution
`chunks_exact` ignores an incomplete final block. In production, decide whether to discard it or pad it with silence. Never pass a non-160-sample slice to `process_audio`.
:::
