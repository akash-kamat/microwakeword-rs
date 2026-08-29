---
title: Runnable examples
description: Run every example shipped with the crate repository.
---

Clone the repository so Cargo can see its `examples/` directory:

```bash
git clone https://github.com/akash-kamat/microwakeword-rs.git
cd microwakeword-rs
```

The sample commands below assume the repository sits beside the provided `models/` folder. Replace paths with your own model files as needed.

| Example | Command |
| --- | --- |
| Full microphone CLI | `cargo run --release --example listen -- ../models/miku.json --cooldown 0.5` |
| List microphones | `cargo run --release --example listen -- --list-devices` |
| Choose an input | `cargo run --release --example listen -- ../models/miku.json --device "USB Mic"` |
| Model without JSON | `cargo run --release --example model_only -- ../models/miku.tflite miku 0.3 3` |
| Process raw PCM | `cargo run --no-default-features --example detect_pcm -- ../models/miku.json audio.raw` |
| Custom TFLite library | `cargo run --no-default-features --example custom_runtime -- CONFIG.json tensorflowlite_c.dll` |
| Inspect/reset detector | `cargo run --release --example detector_access -- ../models/miku.json` |
| Match error variants | `cargo run --release --example error_handling -- ../models/miku.json` |
| Reconnect after mic loss | `cargo run --release --example reconnect -- ../models/miku.json` |

## Main command-line program

The crate also includes a `micro-wakeword` binary:

```bash
cargo run --release --bin micro-wakeword -- ../models/miku.json --cooldown 0.5
```

Discover all flags:

```bash
cargo run --release --bin micro-wakeword -- --help
```

`detect_pcm` expects headerless little-endian i16 mono audio at 16 kHz. Omit its audio path to process generated silence.
