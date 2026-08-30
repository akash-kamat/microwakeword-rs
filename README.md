<div align="center">

# micro-wakeword

**Local wake-word detection for Rust, powered by microWakeWord models.**

[![Crates.io](https://img.shields.io/crates/v/micro-wakeword.svg)](https://crates.io/crates/micro-wakeword)
[![docs.rs](https://docs.rs/micro-wakeword/badge.svg)](https://docs.rs/micro-wakeword)
[![CI](https://github.com/akash-kamat/microwakeword-rs/actions/workflows/ci.yml/badge.svg)](https://github.com/akash-kamat/microwakeword-rs/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

[Handbook](https://akash-kamat.github.io/microwakeword-rs/) · [API docs](https://docs.rs/micro-wakeword) · [Examples](examples) · [Roadmap](ROADMAP.md)

</div>

`micro-wakeword` runs
[microWakeWord](https://github.com/kahrendt/microWakeWord)-compatible
TensorFlow Lite models entirely on-device. Use the ready-made microphone
listener, or feed PCM from your own audio pipeline.

- Local inference: no cloud service or network connection
- Simple microphone API with resampling and channel conversion
- Low-level streaming API for files, sockets, bots, and custom audio engines
- Standard microWakeWord JSON support, plus a model-only builder
- Configurable cooldown, threshold, device, and TensorFlow Lite runtime

```text
Microphone ──> Listener ─┐
                        ├──> Detector ──> Detection { wake_word, probability }
Your 16 kHz PCM ─────────┘
```

> [!NOTE]
> Windows x86-64 is supported out of the box. Other platforms currently need a
> compatible TensorFlow Lite C shared library supplied by the application.

## Install

```powershell
cargo add micro-wakeword
```

Windows builds require the MSVC C++ build tools because the audio frontend is
compiled as part of the crate.

### Ready-made Windows command

Each [GitHub release](https://github.com/akash-kamat/microwakeword-rs/releases)
includes a prebuilt Windows x86-64 executable and its SHA-256 checksum. Keep
your model JSON and `.tflite` file together, then run:

```powershell
.\micro-wakeword-v0.1.2-windows-x86_64.exe wake-word.json --cooldown 0.5
```

Use `--list-devices` to find microphone names and `--help` for every option.

## Quick start: listen to a microphone

```rust,no_run
use micro_wakeword::Listener;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut listener = Listener::from_config("wake-word.json")?;
    println!("Listening for {}", listener.detector().config().wake_word);

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

The JSON file supplies the model path and its recommended detection settings.
Relative model paths are resolved from the JSON file's directory.

### Choose a microphone and cooldown

```rust,no_run
use std::time::Duration;
use micro_wakeword::Listener;

# fn run() -> micro_wakeword::Result<()> {
let mut listener = Listener::config_builder("wake-word.json")?
    .device("USB Mic") // A device index or part of its name.
    .cooldown(Duration::from_millis(500))
    .build()?;
# Ok(()) }
```

The default cooldown is one second. Use `Duration::ZERO` to disable repeat
suppression. List available inputs with `available_input_devices()`.

If processing briefly falls behind, the listener discards stale microphone
audio, resets the detector, and continues automatically. You can inspect the
cumulative count with `listener.dropped_audio_blocks()`; a dropped block is 10
milliseconds. Run latency-sensitive applications with `--release`.

### Handle microphone disconnection

Without `.device(...)`, a listener selects the operating system's default
input **when the listener is created**. It does not silently switch to another
microphone if that device is unplugged. `next_detection()` then returns either
`Error::AudioStreamEnded` or `Error::Audio`, depending on the audio driver.

Applications that should survive an unplug can catch those errors, drop the
old listener, wait briefly, and create a new one. A newly created default-device
listener uses whatever input the operating system now considers the default;
the crate never chooses an arbitrary fallback device. See the complete
[`reconnect` example](examples/reconnect.rs).

```rust,no_run
use std::{thread, time::Duration};
use micro_wakeword::{Error, Listener};

# fn run() -> Result<(), Box<dyn std::error::Error>> {
'connect: loop {
    let mut listener = match Listener::from_config("wake-word.json") {
        Ok(listener) => listener,
        Err(Error::Audio(_) | Error::AudioStreamEnded) => {
            thread::sleep(Duration::from_secs(2));
            continue;
        }
        Err(error) => return Err(error.into()),
    };

    loop {
        match listener.next_detection() {
            Ok(Some(detection)) => println!("Detected {}", detection.wake_word),
            Ok(None) => return Ok(()),
            Err(Error::Audio(_) | Error::AudioStreamEnded) => {
                thread::sleep(Duration::from_secs(2));
                continue 'connect;
            }
            Err(error) => return Err(error.into()),
        }
    }
}
# }
```

## Only have a `.tflite` model?

JSON is recommended, but it is not required. Provide the settings yourself:

```rust,no_run
use micro_wakeword::Listener;

# fn run() -> micro_wakeword::Result<()> {
let mut listener = Listener::builder("hey_computer.tflite")
    .wake_word("hey computer")
    .probability_cutoff(0.5)
    .sliding_window_size(3)
    .build()?;
# Ok(()) }
```

The crate cannot infer the cutoff or sliding-window size from a TFLite file.
Use values supplied by the model author when possible. Otherwise, validate them
against recordings containing the wake word and recordings containing ordinary
speech and background noise.

| Setting | Lower value | Higher value |
| --- | --- | --- |
| `probability_cutoff` | More sensitive; more false activations | Stricter; more missed wake words |
| `sliding_window_size` | Faster; more affected by brief spikes | Steadier; slower to activate |

## Bring your own audio

Use `Detector` when audio already comes from a file, WebSocket, voice bot,
media pipeline, or another capture library:

```rust,no_run
use micro_wakeword::{AUDIO_BLOCK_SAMPLES, Detector};

# fn run() -> micro_wakeword::Result<()> {
let mut detector = Detector::from_config("wake-word.json")?;
let pcm = [0_i16; AUDIO_BLOCK_SAMPLES];

if let Some(detection) = detector.process_audio(&pcm)? {
    println!("Detected {}", detection.wake_word);
}
# Ok(()) }
```

Each call requires exactly **160 samples** of **16 kHz, mono, signed 16-bit
PCM**—10 milliseconds of audio. The low-level detector does not resample,
downmix, or apply a cooldown. Call `reset()` before a new unrelated stream or
after an audio discontinuity.

## Which API should I use?

| You have | Use |
| --- | --- |
| A microphone and model JSON | `Listener::from_config` |
| A microphone and only a TFLite model | `Listener::builder` |
| Your own decoded/resampled audio | `Detector::from_config` |
| Your own audio and only a TFLite model | `Detector::builder` |

## Runnable examples

Commands below use the sample files in this repository's `../models` folder:

| Example | Command |
| --- | --- |
| Microphone, devices, cooldown | `cargo run --release --example listen -- ../models/miku.json --cooldown 0.5` |
| List microphones | `cargo run --release --example listen -- --list-devices` |
| TFLite model without JSON | `cargo run --release --example model_only -- ../models/miku.tflite miku 0.3 3` |
| Process raw PCM directly | `cargo run --no-default-features --example detect_pcm -- ../models/miku.json audio.raw` |
| Supply a TFLite runtime | `cargo run --no-default-features --example custom_runtime -- CONFIG.json tensorflowlite_c.dll` |
| Inspect/reset a listener's detector | `cargo run --release --example detector_access -- ../models/miku.json` |
| Match individual error types | `cargo run --release --example error_handling -- ../models/miku.json` |
| Reconnect after microphone loss | `cargo run --release --example reconnect -- ../models/miku.json` |

`detect_pcm` expects headerless little-endian i16 audio at 16 kHz mono. Omit
`audio.raw` to run it against generated silence.

## TensorFlow Lite runtime

On Windows x86-64, `Runtime::Auto` uses this order:

1. `MICRO_WAKEWORD_TFLITE_LIB`
2. `TFLITE_C_LIB`
3. The bundled, checksum-verified TensorFlow Lite 2.17.1 runtime

To choose a library explicitly:

```rust,no_run
use micro_wakeword::{Detector, Runtime};

# fn run() -> micro_wakeword::Result<()> {
let detector = Detector::from_config_with_runtime(
    "wake-word.json",
    Runtime::from_path("tensorflowlite_c.dll"),
)?;
# Ok(()) }
```

`Runtime::System` searches the platform locations supported by `tflite-c-rs`.

## Model compatibility

Models must use the standard microWakeWord pipeline:

- 16 kHz mono input audio
- 40 frontend features
- Signed int8 input tensor shaped `[1, rows, 40]`
- One int8, uint8, or float32 probability output
- Standard configuration format version 2 when using JSON
- 10 ms feature step

An incompatible model returns a descriptive error while loading.

## Feature flags

The default `listener` feature enables CPAL microphone capture and Rubato
resampling. Applications supplying their own 16 kHz PCM can leave them out:

```toml
micro-wakeword = { version = "0.1", default-features = false }
```

## License

`micro-wakeword` is available under the [MIT License](LICENSE). Bundled
TensorFlow microfrontend and KissFFT sources retain their upstream licenses;
see [LICENSES](LICENSES/THIRD-PARTY.md).
