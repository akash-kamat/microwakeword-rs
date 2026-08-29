---
title: Installation
description: Add micro-wakeword and prepare your platform.
---

## Add the crate

From your Rust project:

```bash
cargo add micro-wakeword
```

That enables the default `listener` feature, which includes microphone capture, channel conversion, and resampling.

For a low-level detector with no microphone dependencies:

```bash
cargo add micro-wakeword --no-default-features
```

Equivalent `Cargo.toml` entries are:

```toml
[dependencies]
micro-wakeword = "0.1"

# Or, when your application supplies 16 kHz mono PCM:
# micro-wakeword = { version = "0.1", default-features = false }
```

## Platform setup

### Windows x86-64

The TensorFlow Lite 2.17.1 runtime is bundled and checksum-verified. Install the **MSVC C++ build tools** because the crate compiles its audio feature frontend from C++ source.

With Rust installed through rustup, the standard Windows MSVC toolchain is recommended:

```powershell
rustup default stable-x86_64-pc-windows-msvc
```

### Linux

Install your distribution's audio development package for CPAL. On Ubuntu/Debian:

```bash
sudo apt-get install libasound2-dev
```

You must also provide a compatible TensorFlow Lite C shared library. See [TensorFlow Lite runtime](../../reference/runtime/).

### macOS

Microphone capture builds through CoreAudio. You must provide a compatible TensorFlow Lite C shared library and macOS will ask for microphone permission when your app first records.

## Keep model files together

A normal project layout looks like this:

```text
my-app/
├── Cargo.toml
├── models/
│   ├── wake-word.json
│   └── wake-word.tflite
└── src/
    └── main.rs
```

The model path inside `wake-word.json` is resolved **relative to the JSON file**, not your executable.

## Check the setup

```bash
cargo check
cargo run --release
```

Use release mode for real listening. Debug builds can process audio too slowly and cause avoidable dropped blocks.

:::tip[Try without writing an app]
GitHub releases include a Windows command-line executable. Put it beside your model files and run `micro-wakeword-…exe wake-word.json`.
:::
