---
title: Model JSON
description: Understand the standard microWakeWord configuration file.
---

The JSON file is the safest way to distribute a model because it carries the settings chosen by its author.

```json
{
  "type": "micro",
  "wake_word": "miku",
  "author": "Model author",
  "website": "https://example.com/model",
  "model": "miku.tflite",
  "trained_languages": ["en"],
  "version": 2,
  "micro": {
    "probability_cutoff": 0.3,
    "sliding_window_size": 3,
    "feature_step_size": 10
  }
}
```

## Fields the crate uses

| Field | Meaning |
| --- | --- |
| `type` | Must be `"micro"` |
| `wake_word` | Human-readable label returned in a detection |
| `model` | `.tflite` path, relative to this JSON unless absolute |
| `version` | Must currently be `2` |
| `micro.probability_cutoff` | Minimum smoothed score needed to detect |
| `micro.sliding_window_size` | Number of recent scores averaged |
| `micro.feature_step_size` | Must be `10` milliseconds |
| `author`, `website`, `trained_languages` | Exposed as model metadata |

Extra JSON fields used by ecosystems such as ESPHome are accepted and ignored.

## Load it directly

```rust
use micro_wakeword::{Config, Detector, Listener};

# fn run() -> micro_wakeword::Result<()> {
let config = Config::from_file("models/miku.json")?;
println!("Model: {}", config.model_path.display());

let detector = Detector::from_config("models/miku.json")?;
let listener = Listener::from_config("models/miku.json")?;
# Ok(()) }
```

`Config::from_file` validates values before the model is loaded. Relative model paths are resolved against the JSON file's directory, so launching the program from another working directory does not break that relationship.
