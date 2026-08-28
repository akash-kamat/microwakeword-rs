use std::fs;

use micro_wakeword::{Config, Error};

#[test]
fn parses_config_and_resolves_relative_model_path() {
    let directory = tempfile::tempdir().unwrap();
    fs::write(directory.path().join("wake.tflite"), []).unwrap();
    let path = directory.path().join("wake.json");
    fs::write(
        &path,
        r#"{
        "type":"micro", "wake_word":"Test", "model":"wake.tflite",
        "version":2, "author":"Author", "trained_languages":["en"],
        "micro":{"probability_cutoff":0.5,"sliding_window_size":3,"feature_step_size":10}
    }"#,
    )
    .unwrap();
    let config = Config::from_file(&path).unwrap();
    assert_eq!(config.model_path, directory.path().join("wake.tflite"));
    assert_eq!(config.wake_word, "Test");
    assert_eq!(config.metadata.author.as_deref(), Some("Author"));
}

#[test]
fn rejects_unsupported_format_version() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("wake.json");
    fs::write(&path, r#"{
        "type":"micro", "wake_word":"Test", "model":"wake.tflite",
        "version":3, "micro":{"probability_cutoff":0.5,"sliding_window_size":3,"feature_step_size":10}
    }"#).unwrap();
    assert!(matches!(
        Config::from_file(path),
        Err(Error::InvalidConfig(_))
    ));
}
