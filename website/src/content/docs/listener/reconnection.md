---
title: Disconnect & reconnect
description: Recover when a microphone disappears and understand default-device behavior.
---

When the active device is unplugged or its driver stops, `next_detection()` returns `Error::AudioStreamEnded` or `Error::Audio(...)`. It does not silently select another microphone.

## Recreate the listener

```rust
use std::{thread, time::Duration};
use micro_wakeword::{Error, Listener};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = "wake-word.json";

    'reconnect: loop {
        let mut listener = match Listener::from_config(config) {
            Ok(listener) => listener,
            Err(error) if is_microphone_error(&error) => {
                eprintln!("Microphone unavailable: {error}");
                thread::sleep(Duration::from_secs(2));
                continue;
            }
            Err(error) => return Err(error.into()),
        };

        loop {
            match listener.next_detection() {
                Ok(Some(hit)) => println!("Detected {}", hit.wake_word),
                Ok(None) => return Ok(()),
                Err(error) if is_microphone_error(&error) => {
                    eprintln!("Microphone lost: {error}");
                    thread::sleep(Duration::from_secs(2));
                    continue 'reconnect;
                }
                Err(error) => return Err(error.into()),
            }
        }
    }
}

fn is_microphone_error(error: &Error) -> bool {
    matches!(error, Error::Audio(_) | Error::AudioStreamEnded)
}
```

Recreating a listener with no `.device(...)` asks the OS for its **current** default input. The two-second delay prevents a tight retry loop and gives drivers time to finish registering a newly connected device.

## What this does—and does not—follow

| Event | Existing listener | Reconnect loop |
| --- | --- | --- |
| Active mic is unplugged | Reports an audio error | Creates a listener for the current default |
| Same mic is replugged after failure | Old stream is dead | A later retry can open it if it is default |
| User changes default while old stream is healthy | Continues using old mic | No error means no rebuild |

To follow every healthy default-device change, the application needs platform notifications (for example, Windows `IMMNotificationClient`) and must deliberately rebuild its audio stream. That policy is outside the portable listener API.

:::note
The retry sleeps while waiting; it is not a busy loop. Its CPU and memory impact is negligible.
:::
