---
title: Bring your own audio
description: Feed audio from files, calls, sockets, bots, or another capture library.
---

`Detector` is independent of microphones. It only sees PCM samples, so the audio may come from anywhere.

## Required format

Every call to `process_audio` must contain:

| Property | Required value |
| --- | --- |
| Sample rate | 16,000 Hz |
| Channels | 1 (mono) |
| Sample type | signed 16-bit PCM (`i16`) |
| Block size | 160 samples |
| Time represented | 10 ms |

The constants `SAMPLE_RATE` and `AUDIO_BLOCK_SAMPLES` avoid magic numbers.

```rust
use micro_wakeword::{AUDIO_BLOCK_SAMPLES, Detector};

# fn next_block() -> [i16; AUDIO_BLOCK_SAMPLES] { [0; AUDIO_BLOCK_SAMPLES] }
fn main() -> micro_wakeword::Result<()> {
    let mut detector = Detector::from_config("wake-word.json")?;

    loop {
        let block: [i16; AUDIO_BLOCK_SAMPLES] = next_block();
        if let Some(hit) = detector.process_audio(&block)? {
            println!("Detected {}", hit.wake_word);
        }
    }
}
```

## Where app audio comes from

Examples include:

- a WebRTC or voice-call SDK delivering decoded frames;
- a Discord/voice bot receiving Opus packets, then decoding them;
- a browser sending audio over WebSocket;
- an FFmpeg/GStreamer decoding pipeline;
- a game engine's audio capture callback;
- an embedded or network microphone;
- another Rust capture crate;
- a WAV or raw PCM file.

For compressed data, decode it first. For stereo or another sample rate, downmix and resample before calling the detector.

```text
WebSocket bytes → decode Opus → downmix → resample to 16 kHz → chunks of 160 i16 → Detector
```

## Backpressure and boundaries

Preserve chronological order and do not fabricate continuity after dropped data. If blocks are lost, a call ends, or you switch speakers, call `detector.reset()` before continuing.

The detector has no cooldown. Low-level callers should use media timestamps or their own state machine if repeated detections need suppression.
