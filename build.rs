use std::path::PathBuf;

fn main() {
    let vendor = PathBuf::from("native/vendor");
    let frontend = vendor.join("tensorflow/lite/experimental/microfrontend/lib");
    let kissfft = vendor.join("kissfft");
    let sources = [
        "kiss_fft_int16.cc",
        "fft.cc",
        "fft_util.cc",
        "filterbank.cc",
        "filterbank_util.cc",
        "frontend.cc",
        "frontend_util.cc",
        "log_lut.cc",
        "log_scale.cc",
        "log_scale_util.cc",
        "noise_reduction.cc",
        "noise_reduction_util.cc",
        "pcan_gain_control.cc",
        "pcan_gain_control_util.cc",
        "window.cc",
        "window_util.cc",
    ];

    let mut build = cc::Build::new();
    build
        .cpp(true)
        .define("FIXED_POINT", "16")
        .include(&vendor)
        .include(&kissfft)
        .file("native/frontend_bridge.cc")
        .file(kissfft.join("kiss_fft.cc"))
        .file(kissfft.join("tools/kiss_fftr.cc"))
        .warnings(false);
    for source in sources {
        build.file(frontend.join(source));
    }
    build.compile("micro_wakeword_frontend");
    println!("cargo:rerun-if-changed=native");
}
