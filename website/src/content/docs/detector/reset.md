---
title: Reset & stream boundaries
description: Clear temporal state after gaps or when changing audio sources.
---

Wake-word detection is stateful. The frontend retains context and the detector retains model input rows and recent probabilities across calls.

Call `reset()` when continuity is broken:

```rust
detector.reset()?;
```

## Reset after

- dropped or missing audio blocks;
- seeking within a file;
- switching callers, speakers, tracks, or input devices;
- restarting a decoder;
- starting an unrelated recording;
- a network discontinuity.

Without a reset, the beginning of one source may be evaluated using feature history from another source. That can distort probabilities or create false activations near the boundary.

## Do not reset between normal blocks

```rust
// Correct: state flows across contiguous blocks.
for block in contiguous_audio.chunks_exact(AUDIO_BLOCK_SAMPLES) {
    detector.process_audio(block)?;
}

// Boundary: clear state once before an unrelated stream.
detector.reset()?;
```

`Listener` manages this automatically when it detects an internal queue overflow. Low-level `Detector` callers own this responsibility because only they know whether their input remains continuous.
