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

The TensorFlow Lite runtime is bundled and checksum-verified. Install the **MSVC C++ build tools** because the crate compiles its audio feature frontend from C++ source.

With Rust installed through rustup, the standard Windows MSVC toolchain is recommended:

```powershell
rustup default stable-x86_64-pc-windows-msvc
```

### Linux x86-64 and ARM64

Install your distribution's audio development package for CPAL. On Ubuntu/Debian:

```bash
sudo apt-get install libasound2-dev
```

Also install a C++ compiler if your distribution does not include one. The
TensorFlow Lite runtime is bundled for glibc-based x86-64 and ARM64 systems.
Musl and other architectures can use an explicit or system runtime; see
[TensorFlow Lite runtime](../../reference/runtime/).

### macOS Intel and Apple Silicon

Install Xcode Command Line Tools with `xcode-select --install`. Microphone
capture uses CoreAudio, the matching TensorFlow Lite runtime is bundled, and
macOS asks for microphone permission when your app first records.

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
GitHub releases include command-line executables for every bundled target. Put one beside your model files and run it with `wake-word.json`.
:::
