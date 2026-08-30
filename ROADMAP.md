# micro-wakeword Roadmap

`micro-wakeword` 0.1 provides the core listener and detector APIs. The items
below are the main steps toward broader compatibility and production maturity.

## Recognition confidence

- [ ] Add automated end-to-end recognition tests using real positive wake-word
      recordings and negative speech/background recordings.
- [ ] Test against a larger collection of microWakeWord models, including
      different phrases, input-window sizes, and output tensor types.

## Reliability

- [ ] Add long-running microphone, inference, CPU, and memory tests to catch
      leaks, stalls, and queue behaviour that short tests cannot expose.
- [ ] Add automated microphone tests for unplugging, reconnecting, device
      removal, and operating-system default-input changes where the platform
      test environment permits it.

## Platform support

- [x] Provide Linux and macOS CI that loads the real bundled runtime rather
      than stopping at compile-only checks.
- [x] Bundle verified TensorFlow Lite C runtimes for Linux x86-64/ARM64 and
      macOS Intel/Apple Silicon, with checksums and license notices.

## Package and runtime footprint

Current behaviour:

- The default `listener` feature enables CPAL microphone capture and Rubato
  resampling. CPAL compiles only the audio backend for the current platform:
  WASAPI on Windows, ALSA on Linux, or CoreAudio on macOS.
- `default-features = false` removes microphone capture and resampling for
  applications that supply their own 16 kHz mono `i16` PCM audio to
  `Detector`.
- The crates.io source package currently contains every bundled TensorFlow
  Lite runtime, so all of them consume download and Cargo-cache disk space.
  Conditional compilation embeds and loads only the current target's runtime,
  so the other platform binaries do not consume application RAM.

Planned improvements:

- [ ] Move bundled native libraries into internal, target-specific runtime
      crates before the main package approaches crates.io's size limit. The
      public installation remains `cargo add micro-wakeword`; Cargo selects
      the appropriate internal runtime dependency automatically.
- [ ] Publish repeatable memory benchmarks for startup, steady-state
      detection, and long-running use with both the default `Listener` and the
      low-level `Detector` configuration. Track regressions in CI where
      measurements are stable enough to be meaningful.

## Model compatibility

- [ ] Support more model/input formats beyond configuration version 2 with a
      signed int8 `[1, rows, 40]` input, when real microWakeWord models require
      them.
- [ ] Keep compatibility checks explicit and return actionable errors instead
      of guessing how an unknown model should be processed.

These are roadmap goals, not promises for a particular release. New formats
and platforms should be added only with representative models and automated
tests.
