use std::path::PathBuf;
use std::time::Duration;

use micro_wakeword::{Listener, Runtime, available_input_devices};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut config = None;
    let mut device = None;
    let mut runtime = None;
    let mut cooldown = None;
    let mut list_devices = false;
    let mut args = std::env::args_os().skip(1);
    while let Some(arg) = args.next() {
        match arg.to_string_lossy().as_ref() {
            "--device" => {
                device = Some(
                    args.next()
                        .ok_or("--device needs a name or index")?
                        .to_string_lossy()
                        .into_owned(),
                )
            }
            "--tflite-lib" => {
                runtime = Some(PathBuf::from(
                    args.next().ok_or("--tflite-lib needs a path")?,
                ))
            }
            "--cooldown" => {
                let seconds: f64 = args
                    .next()
                    .ok_or("--cooldown needs a number of seconds")?
                    .to_string_lossy()
                    .parse()?;
                if !seconds.is_finite() || seconds < 0.0 {
                    return Err("--cooldown must be a finite, non-negative number".into());
                }
                cooldown = Some(Duration::from_secs_f64(seconds));
            }
            "--list-devices" => list_devices = true,
            "-h" | "--help" => {
                println!(
                    "listen CONFIG.json [--device INDEX|NAME] [--cooldown SECONDS] \
                     [--tflite-lib PATH]\nlisten --list-devices"
                );
                return Ok(());
            }
            value if value.starts_with('-') => {
                return Err(format!("unknown argument: {value}").into());
            }
            _ if config.is_none() => config = Some(PathBuf::from(arg)),
            value => return Err(format!("unexpected argument: {value}").into()),
        }
    }
    if list_devices {
        for input in available_input_devices()? {
            println!("{}: {}", input.index, input.name);
        }
        return Ok(());
    }
    let config = config.ok_or("a microWakeWord JSON path is required")?;
    let mut builder = Listener::config_builder(&config)?;
    if let Some(runtime) = runtime {
        builder = builder.runtime(Runtime::from_path(runtime));
    }
    if let Some(device) = device {
        builder = builder.device(device);
    }
    if let Some(cooldown) = cooldown {
        builder = builder.cooldown(cooldown);
    }
    let mut listener = builder.build()?;
    println!(
        "Listening for {} with a {:.3}s cooldown. Press Ctrl+C to stop.",
        listener.detector().config().wake_word,
        listener.cooldown().as_secs_f64()
    );
    while let Some(detection) = listener.next_detection()? {
        println!(
            "Detected {} ({:.1}%)",
            detection.wake_word,
            detection.probability * 100.0
        );
    }
    Ok(())
}
