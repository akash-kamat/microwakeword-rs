---
title: Configuration reference
description: Public configuration types, builders, constants, and metadata.
---

## Constants

| Constant | Value | Meaning |
| --- | --- | --- |
| `SAMPLE_RATE` | `16_000` | Required low-level sample rate |
| `AUDIO_BLOCK_SAMPLES` | `160` | Samples passed per detector call |
| `DEFAULT_COOLDOWN` | `1 second` | Listener repeat-suppression period |

## `Config`

`Config::from_file(path)` parses standard JSON and exposes:

| Field | Type |
| --- | --- |
| `model_path` | `PathBuf` |
| `wake_word` | `String` |
| `probability_cutoff` | `f32` |
| `sliding_window_size` | `usize` |
| `feature_step_size_ms` | `u32` |
| `metadata` | `ModelMetadata` |

Metadata contains optional author and website values, trained languages, and format version.

## `DetectorBuilder`

Start with `Detector::builder(model_path)`. A model-only build requires all three model-specific settings:

```rust
# use micro_wakeword::Detector;
# fn run() -> micro_wakeword::Result<()> {
let detector = Detector::builder("model.tflite")
    .wake_word("hello")
    .probability_cutoff(0.5)
    .sliding_window_size(3)
    .feature_step_size_ms(10)
    .build()?;
# Ok(()) }
```

`.feature_step_size_ms(10)` is optional because `10` is the only supported value and the default. `.runtime(...)` is also optional and defaults to `Runtime::Auto`.

## `ListenerBuilder`

There are two entry points:

```rust
// Parse model values from JSON; builder methods can override them.
let builder = micro_wakeword::Listener::config_builder("wake-word.json")?;

// Standalone model; wake word, cutoff, and window become required.
let builder = micro_wakeword::Listener::builder("model.tflite");
# Ok::<(), micro_wakeword::Error>(())
```

Methods:

| Method | Purpose |
| --- | --- |
| `.wake_word(...)` | Set or override the reported label |
| `.probability_cutoff(...)` | Set or override detection threshold |
| `.sliding_window_size(...)` | Set or override score smoothing |
| `.runtime(...)` | Choose TensorFlow Lite loading policy |
| `.device(...)` | Select an input name or index |
| `.cooldown(Duration)` | Set repeat suppression; default 1 second |

For exact signatures and trait details, use the generated [API documentation](https://docs.rs/micro-wakeword/latest/micro_wakeword/).
