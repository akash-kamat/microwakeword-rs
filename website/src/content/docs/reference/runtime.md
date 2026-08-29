---
title: TensorFlow Lite runtime
description: Understand automatic, explicit, and system runtime loading.
---

The `.tflite` model needs the TensorFlow Lite **C shared library** at runtime. `Runtime` controls where it comes from.

## `Runtime::Auto`

This is the default. It checks in order:

1. `MICRO_WAKEWORD_TFLITE_LIB` environment variable;
2. `TFLITE_C_LIB` environment variable;
3. the bundled, checksum-verified TensorFlow Lite 2.17.1 runtime on Windows x86-64.

On unsupported bundled-runtime platforms, supply a path or choose the system loader.

## Explicit path

```rust
use micro_wakeword::{Detector, Runtime};

# fn run() -> micro_wakeword::Result<()> {
let detector = Detector::from_config_with_runtime(
    "wake-word.json",
    Runtime::from_path("tensorflowlite_c.dll"),
)?;
# Ok(()) }
```

With a listener builder:

```rust
# use micro_wakeword::{Listener, Runtime};
# fn run() -> micro_wakeword::Result<()> {
let listener = Listener::config_builder("wake-word.json")?
    .runtime(Runtime::from_path("/opt/tensorflow/libtensorflowlite_c.so"))
    .build()?;
# Ok(()) }
```

## `Runtime::System`

```rust
# use micro_wakeword::{Detector, Runtime};
# fn run() -> micro_wakeword::Result<()> {
let detector = Detector::builder("model.tflite")
    .wake_word("hello")
    .probability_cutoff(0.5)
    .sliding_window_size(3)
    .runtime(Runtime::System)
    .build()?;
# Ok(()) }
```

This asks the underlying `tflite-c-rs` loader to search its supported system locations.

## Why allow a custom path?

It lets applications support non-Windows platforms, control how native dependencies are packaged, use an approved runtime build, or test compatibility with another TensorFlow Lite release. The path points to the native library—not the model.
