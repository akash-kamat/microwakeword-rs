---
title: Performance & dropped audio
description: Keep real-time capture healthy and understand queue overflow recovery.
---

Wake-word detection is a real-time stream: ten milliseconds of new audio arrives every ten milliseconds. The application needs to process blocks at least as quickly as they arrive on average.

## Always test a release build

```bash
cargo run --release
```

Debug-mode Rust and unoptimized native code can be much slower. A debug result is useful for correctness, not latency or CPU measurements.

## Queue overflow behavior

The capture callback never blocks waiting for inference. If processing falls behind and the internal queue fills, `Listener`:

1. drops incoming blocks instead of blocking the audio driver;
2. drains stale queued audio;
3. resets detector state because the stream now has a gap;
4. continues listening.

This recovery is automatic. Inspect the cumulative count when diagnosing a slow system:

```rust
# use micro_wakeword::Listener;
# fn run() -> micro_wakeword::Result<()> {
let mut listener = Listener::from_config("wake-word.json")?;

while let Some(hit) = listener.next_detection()? {
    println!("Detected {}", hit.wake_word);
    println!("Dropped blocks: {}", listener.dropped_audio_blocks());
}
# Ok(()) }
```

Each block represents 10 ms. Occasional recovery is survivable, but a continually increasing count means the machine cannot keep up or the detection thread is being starved.

## Keep the detection loop responsive

Do not perform long work directly inside the detection loop. Send events to another thread or asynchronous task:

```rust
// Conceptual pattern: hand off the event quickly.
while let Some(hit) = listener.next_detection()? {
    event_sender.send(hit)?;
}
```

Also avoid running several heavy models on a constrained CPU without measuring their combined real-time cost.
