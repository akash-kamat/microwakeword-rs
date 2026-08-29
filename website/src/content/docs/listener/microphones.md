---
title: Select a microphone
description: List inputs, use the default, or select one explicitly.
---

## Use the current default

Omit `.device(...)` and the listener asks the operating system for its default input during construction:

```rust
let listener = micro_wakeword::Listener::from_config("wake-word.json")?;
# Ok::<(), micro_wakeword::Error>(())
```

This is a one-time selection. Changing the system default later does not move the live stream.

## List available inputs

```rust
use micro_wakeword::available_input_devices;

fn main() -> micro_wakeword::Result<()> {
    for device in available_input_devices()? {
        println!("{}: {}", device.index, device.name);
    }
    Ok(())
}
```

Indices are convenient for a command-line choice made immediately after listing. Device names are easier to save, but drivers may rename them after an update or when plugged into another port.

## Select by index or name

```rust
use micro_wakeword::Listener;

# fn run() -> micro_wakeword::Result<()> {
let by_index = Listener::config_builder("wake-word.json")?
    .device("1")
    .build()?;

let by_name = Listener::config_builder("wake-word.json")?
    .device("USB Microphone")
    .build()?;
# Ok(()) }
```

Name matching is case-insensitive and accepts an unambiguous substring. If multiple devices match, use the full name or an index.

## Device policy belongs to the application

The library opens the device you request. Your app decides whether it should:

- remain fixed to a chosen microphone;
- reconnect to the current system default after failure;
- expose a device selector;
- follow operating-system default-device notifications.

That distinction avoids a voice app unexpectedly recording from a different microphone without the user's chosen policy.
