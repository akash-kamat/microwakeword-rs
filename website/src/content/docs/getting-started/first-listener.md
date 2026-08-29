---
title: Your first listener
description: Detect a wake word from the system's default microphone.
---

`Listener` is the batteries-included API. It opens a microphone, converts its audio to the format the model expects, owns a `Detector`, and suppresses immediate repeats.

## 1. Add the code

```rust
use micro_wakeword::Listener;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut listener = Listener::from_config("models/wake-word.json")?;

    println!(
        "Listening for {}",
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
```

## 2. Run in release mode

```bash
cargo run --release
```

Speak the model's wake phrase. A detection contains:

- `wake_word`: the label from your configuration;
- `probability`: the smoothed model score from `0.0` to `1.0`.

## What each call does

1. `Listener::from_config(...)` parses and validates JSON, resolves its model path, loads TensorFlow Lite, opens the current default input, and starts the audio stream.
2. `next_detection()` waits for microphone audio and processes it until a detection occurs or the stream ends.
3. The listener applies its default **one-second cooldown** after a detection.
4. The `while let` loop continues until `next_detection()` returns `Ok(None)`. Microphone failures are returned as errors rather than silently hidden.

:::caution[“Default” is selected once]
The operating system's default microphone is selected when the listener is created. An existing stream does not migrate when the system default later changes. See [Disconnect & reconnect](../../listener/reconnection/).
:::

## Configure it with a builder

```rust
use std::time::Duration;
use micro_wakeword::Listener;

# fn run() -> micro_wakeword::Result<()> {
let mut listener = Listener::config_builder("models/wake-word.json")?
    .device("USB Mic")
    .cooldown(Duration::from_secs_f64(0.5))
    .build()?;
# Ok(())
```

The device can be its numeric index, its exact name, or an unambiguous case-insensitive part of its name.
