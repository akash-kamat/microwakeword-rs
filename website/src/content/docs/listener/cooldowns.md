---
title: Cooldowns
description: Suppress repeated detections after a wake word fires.
---

A wake-word model evaluates a rolling audio window. The phrase can remain in that window long enough to trigger repeatedly. `Listener` avoids that with a cooldown.

## Default behavior

The default is one second:

```rust
use micro_wakeword::DEFAULT_COOLDOWN;

assert_eq!(DEFAULT_COOLDOWN.as_secs_f64(), 1.0);
```

During the cooldown, the listener keeps consuming audio but does not return another detection. This prevents old audio from accumulating.

## Use decimal seconds

Rust's `Duration` supports fractional seconds:

```rust
use std::time::Duration;
use micro_wakeword::Listener;

# fn run() -> micro_wakeword::Result<()> {
let listener = Listener::config_builder("wake-word.json")?
    .cooldown(Duration::from_secs_f64(0.5))
    .build()?;
# Ok(()) }
```

You can also use `Duration::from_millis(500)`.

## Disable it

```rust
# use micro_wakeword::Listener;
# fn run() -> micro_wakeword::Result<()> {
let listener = Listener::config_builder("wake-word.json")?
    .cooldown(std::time::Duration::ZERO)
    .build()?;
# Ok(()) }
```

This is useful for experiments, probability logging, or when the application already has a state machine controlling repeat activations.

## Why `Detector` has no cooldown

`Detector` processes supplied audio without knowing wall-clock policy. A file might be processed faster than real time, and a voice pipeline might use media timestamps rather than the computer clock. Apply your own cooldown around low-level detections when needed.
