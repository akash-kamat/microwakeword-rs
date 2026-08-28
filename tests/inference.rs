use micro_wakeword::{AUDIO_BLOCK_SAMPLES, Config, Detector, Runtime};

#[test]
fn loads_example_model_and_resets_when_test_paths_are_configured() {
    let Some(config) = std::env::var_os("MICRO_WAKEWORD_TEST_CONFIG") else {
        return;
    };
    let runtime = std::env::var_os("MICRO_WAKEWORD_TEST_RUNTIME")
        .map(Runtime::from_path)
        .unwrap_or_default();
    let parsed = Config::from_file(&config).unwrap();
    let mut detector = Detector::from_config_with_runtime(&config, runtime.clone()).unwrap();
    for _ in 0..12 {
        assert!(
            detector
                .process_audio(&[0; AUDIO_BLOCK_SAMPLES])
                .unwrap()
                .is_none()
        );
    }
    detector.reset().unwrap();

    let built = Detector::builder(parsed.model_path)
        .wake_word(&parsed.wake_word)
        .probability_cutoff(parsed.probability_cutoff)
        .sliding_window_size(parsed.sliding_window_size)
        .runtime(runtime)
        .build()
        .unwrap();
    assert_eq!(built.config().wake_word, parsed.wake_word);
}
