---
title: Threshold & sliding window
description: Tune sensitivity, false activations, and response stability.
---

The model emits a score for each inference window. Two settings turn that noisy stream into a detection.

## Probability cutoff

`probability_cutoff` ranges from `0.0` to `1.0`. A detection requires the smoothed probability to reach this value.

| Lower cutoff | Higher cutoff |
| --- | --- |
| More sensitive | More selective |
| Can hear weaker pronunciations | Rejects more uncertain audio |
| More false activations | More missed or difficult activations |

```rust
# use micro_wakeword::Detector;
# fn run() -> micro_wakeword::Result<()> {
let detector = Detector::builder("model.tflite")
    .wake_word("hello")
    .probability_cutoff(0.42)
    .build()?;
# Ok(()) }
```

## Sliding-window size

The detector averages the latest model probabilities. `sliding_window_size` controls how many are included.

| Smaller window | Larger window |
| --- | --- |
| Reacts more quickly | Produces a steadier score |
| More affected by brief spikes | Requires confidence to persist |
| May increase false activations | May add latency or miss short peaks |

It must be at least `1`.

## Tune them together

A threshold does not have the same meaning under every window size. Changing smoothing changes the score distribution, so evaluate combinations rather than choosing each setting independently.

1. Start with the model author's values.
2. Build representative positive and negative recordings.
3. Measure missed wake words and false activations separately.
4. Adjust one combination at a time.
5. Validate on microphones, speakers, accents, distances, and noise not used during tuning.

The `feature_step_size_ms` must remain `10`; the crate's frontend and streaming contract are designed around ten-millisecond steps.
