---
title: Model without JSON
description: Configure a standalone TFLite model explicitly.
---

JSON is recommended but not required. If you only have a `.tflite` model, use a builder and provide the missing settings.

## Microphone listener

```rust
use micro_wakeword::Listener;

# fn run() -> micro_wakeword::Result<()> {
let mut listener = Listener::builder("hey_computer.tflite")
    .wake_word("hey computer")
    .probability_cutoff(0.5)
    .sliding_window_size(3)
    .build()?;
# Ok(()) }
```

## Low-level detector

```rust
use micro_wakeword::Detector;

# fn run() -> micro_wakeword::Result<()> {
let mut detector = Detector::builder("hey_computer.tflite")
    .wake_word("hey computer")
    .probability_cutoff(0.5)
    .sliding_window_size(3)
    .build()?;
# Ok(()) }
```

## Does the crate guess values?

No. A TFLite file describes tensors and operations, but standard microWakeWord cutoffs and smoothing choices are distribution metadata, not reliably recoverable settings. The builder requires the wake-word name, cutoff, and sliding-window size; it does not invent them. Only the fixed 10 ms feature step and automatic runtime selection have defaults.

Prefer values from the model author or its original JSON. If neither exists, test candidate values against:

- many recordings containing the wake phrase;
- similar phrases that should not trigger;
- ordinary conversations;
- silence, music, television, and expected background noise;
- the actual microphones and rooms where the app will run.

:::caution
Trial and error with only your own voice can produce a configuration that looks good in one room but fails for real users. Treat tuning as an evaluation exercise, not a single successful test.
:::
