---
title: Feature flags & platforms
description: Build only the layer you need and understand native requirements.
---

## Features

The default `listener` feature enables:

- CPAL input-device capture;
- channel downmixing;
- Rubato sample-rate conversion;
- `Listener`, `ListenerBuilder`, `AudioDevice`, and `available_input_devices`.

Disable it when the application supplies correctly formatted audio:

```toml
[dependencies]
micro-wakeword = { version = "0.1", default-features = false }
```

`Detector`, model parsing, feature extraction, inference, and runtime selection remain available.

## Platform matrix

| Platform | Compiles | Microphone backend | Bundled TFLite runtime |
| --- | --- | --- | --- |
| Windows x86-64 | Yes | WASAPI through CPAL | Yes, TFLite 2.17.1 |
| Linux x86-64 (glibc) | Yes | ALSA through CPAL | Yes, TFLite 2.17.1 |
| Linux ARM64 (glibc) | Yes | ALSA through CPAL | Yes, TFLite 2.17.1 |
| macOS Apple Silicon | Yes | CoreAudio through CPAL | Yes, TFLite 2.17.1 |
| macOS Intel | Yes | CoreAudio through CPAL | Yes, TFLite 2.17.0 |

The target-specific library is embedded, extracted into the user's OS cache,
verified by SHA-256, and loaded by absolute path. Unsupported targets need
`Runtime::Path`, `Runtime::System`, or an environment override. Linux binaries
target glibc; musl is not currently bundled.

## Build requirements

The crate compiles the TensorFlow microfrontend and KissFFT C++ sources. A working C/C++ compiler toolchain is required even when the microphone feature is disabled.

Platform support describes current build and runtime wiring, not a guarantee
that every audio device or driver exposes a supported stream configuration.
Handle `Error::Audio` when opening user hardware.
