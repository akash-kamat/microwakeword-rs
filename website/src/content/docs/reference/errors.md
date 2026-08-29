---
title: Errors
description: Match configuration, model, runtime, and microphone failures.
---

Library functions return `micro_wakeword::Result<T>`, an alias for `Result<T, micro_wakeword::Error>`.

| Variant | Meaning | Usually recoverable? |
| --- | --- | --- |
| `Io { path, source }` | Could not read a config/model-related file | After correcting path or permissions |
| `Json { path, source }` | Configuration is not valid JSON | After fixing the file |
| `InvalidConfig(message)` | Values violate the supported contract | After changing configuration |
| `IncompatibleModel(message)` | Tensor shape/type is unsupported | Use a compatible model |
| `Tflite(error)` | Native runtime or inference failed | Depends on native error |
| `UnsupportedPlatform(message)` | No usable runtime on this platform | Supply a runtime |
| `Audio(message)` | Device open, format, or stream failure | Often; device may become available |
| `AudioStreamEnded` | Capture channel ended | Often; recreate listener |

## Match errors deliberately

```rust
use micro_wakeword::{Error, Listener};

fn open() {
    match Listener::from_config("wake-word.json") {
        Ok(_) => println!("Ready"),
        Err(Error::Audio(message)) => eprintln!("Microphone: {message}"),
        Err(Error::AudioStreamEnded) => eprintln!("Microphone stream ended"),
        Err(Error::InvalidConfig(message)) => eprintln!("Settings: {message}"),
        Err(error) => eprintln!("Could not start: {error}"),
    }
}
```

Applications normally retry only transient microphone failures. Retrying malformed JSON or an incompatible model forever hides a deployment problem and wastes resources.

For command-line tools, returning `Result<(), Box<dyn Error>>` from `main` is concise. User-facing apps should translate variants into clear UI and preserve the source error in logs.
