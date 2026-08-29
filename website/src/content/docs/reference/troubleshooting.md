---
title: Troubleshooting
description: Diagnose common setup, audio, model, and runtime problems.
---

## Detections fire constantly

- Confirm the model and JSON belong together.
- Increase `probability_cutoff` or `sliding_window_size` carefully.
- Use the listener cooldown for repeated matches of the same utterance.
- Evaluate with negative recordings, not only live intuition.

## Wake words are missed

- Run an optimized build with `cargo run --release`.
- Confirm the selected microphone and its input level.
- Start from model-author settings.
- Lower the cutoff or window only after measuring false activations.

## `AudioStreamEnded`

The device or capture channel ended. Drop and recreate the listener. If no device was selected explicitly, the new listener asks the OS for its current default. See [Disconnect & reconnect](../../listener/reconnection/).

## Dropped audio blocks keep increasing

The process is not consuming audio in real time. Use release mode, hand expensive work to another thread, and inspect system CPU pressure. The listener automatically drains stale blocks and resets after a gap.

## “Expected exactly 160 samples”

`Detector::process_audio` accepts one 10 ms block per call. Re-buffer your decoded stream into exactly 160-sample chunks. Also verify 16 kHz, mono, signed i16 PCM.

## JSON loads but the model does not

The `model` path is relative to the JSON file. Check that the `.tflite` file exists there and that its tensor contract is [compatible](../../models/compatibility/).

## TensorFlow Lite cannot be loaded

Windows x86-64 uses the bundled runtime unless an environment variable overrides it. On other platforms, supply a compatible shared library with `Runtime::from_path(...)` or configure the system loader.

## Device name is ambiguous

Call `available_input_devices()`, then use the full name or the index displayed in the current run.
