---
title: Choose your API
description: Decide between Listener and Detector.
---

The library has two layers. Neither is “more accurate”; they feed the same detector. The difference is who owns the audio pipeline.

| Your input | Recommended API | What it handles |
| --- | --- | --- |
| System microphone + JSON | `Listener::from_config` | Capture, downmix, resample, detection, cooldown |
| System microphone + `.tflite` only | `Listener::builder` | Same audio work; you supply model settings |
| Your own 16 kHz PCM + JSON | `Detector::from_config` | Features, inference, smoothing, threshold |
| Your own 16 kHz PCM + `.tflite` only | `Detector::builder` | Detection; you supply model settings |

## `Listener`: own the result, not the audio plumbing

Use it for desktop tools, simple assistants, prototypes, and anything that should listen directly to one input device.

```rust
let mut listener = micro_wakeword::Listener::from_config("wake-word.json")?;
while let Some(hit) = listener.next_detection()? {
    println!("{}", hit.wake_word);
}
# Ok::<(), micro_wakeword::Error>(())
```

## `Detector`: own the audio pipeline

Use it when audio already exists inside your application: a decoded file, WebRTC call, game engine, voice bot, browser stream forwarded over a socket, or another capture library.

```rust
use micro_wakeword::{AUDIO_BLOCK_SAMPLES, Detector};

# fn run(block: [i16; AUDIO_BLOCK_SAMPLES]) -> micro_wakeword::Result<()> {
let mut detector = Detector::from_config("wake-word.json")?;
if let Some(hit) = detector.process_audio(&block)? {
    println!("{}", hit.wake_word);
}
# Ok(()) }
```

The low-level API expects exactly 160 signed 16-bit mono samples at 16 kHz per call. It intentionally does **not** resample, downmix, capture, buffer, or apply a cooldown.

## Mix the layers

You can construct a detector yourself and hand it to a listener:

```rust
use micro_wakeword::{Detector, Listener};

# fn run() -> micro_wakeword::Result<()> {
let detector = Detector::from_config("wake-word.json")?;
let listener = Listener::from_detector(detector, Some("USB Mic"))?;
# Ok(()) }
```

This is useful when detector construction needs custom logic but microphone capture should remain automatic.
