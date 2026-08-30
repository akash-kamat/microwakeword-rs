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

## Model compatibility

- [ ] Support more model/input formats beyond configuration version 2 with a
      signed int8 `[1, rows, 40]` input, when real microWakeWord models require
      them.
- [ ] Keep compatibility checks explicit and return actionable errors instead
      of guessing how an unknown model should be processed.

These are roadmap goals, not promises for a particular release. New formats
and platforms should be added only with representative models and automated
tests.
