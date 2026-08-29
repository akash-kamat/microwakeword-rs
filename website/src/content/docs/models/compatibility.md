---
title: Model compatibility
description: Tensor shapes, quantization, frontend, and validation requirements.
---

Compatible models use the standard microWakeWord audio pipeline.

## Required contract

- 16 kHz mono input audio;
- TensorFlow Lite model with exactly one input and one output;
- signed int8 input shaped `[1, rows, 40]`;
- one int8, uint8, or float32 probability output;
- standard configuration format version 2 when JSON is used;
- 10 ms feature step;
- the TensorFlow microfrontend-compatible 40-feature representation.

The number of input rows determines how much feature history the model sees. The detector discovers this dimension from the model.

## Validation errors

Model loading checks tensor count, shape, and supported data types. Failures return `Error::IncompatibleModel` with the expected and actual shape or type where possible.

Configuration errors—unsupported version, invalid cutoff, zero sliding window, wrong step size—return `Error::InvalidConfig` before listening starts.

## What “microWakeWord-compatible” does not mean

A generic speech or audio-classification `.tflite` file is not automatically compatible. It must have been trained for the same frontend features, timing, tensor layout, and output meaning.

If a model uses raw waveform input, MFCCs, a different sample rate, multiple output classes, or float input features, it needs a different preprocessing/inference adapter.
