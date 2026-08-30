# Bundled TensorFlow Lite C runtimes

`Runtime::Auto` extracts the library matching the current operating system and
CPU architecture into the user's cache directory, verifies its SHA-256 digest,
and loads it by absolute path.

| Rust target | Runtime | File SHA-256 | Upstream asset |
| --- | --- | --- | --- |
| `x86_64-pc-windows-msvc` | 2.17.1 | `882e6d8f9866ff84f23d4b964c145b7f0f0a8907fa830dcd8c499e7c46bf3365` | `tflite_c_v2.17.1_windows_amd64.zip` |
| `x86_64-unknown-linux-gnu` | 2.17.1 | `25465edb5cd7aadd00249d4d28f1d922ce5cc90195ad4403458111dd63493bae` | `tflite_c_v2.17.1_linux_amd64.tar.gz` |
| `aarch64-unknown-linux-gnu` | 2.17.1 | `12062437bfde367b1be592ebc4a9fe64b8453f90b3b7dc21fade002c38e1ac1a` | `tflite_c_v2.17.1_linux_arm64.tar.gz` |
| `aarch64-apple-darwin` | 2.17.1 | `e77597b3710e43f58f1c37a9c8979a82901ed9f3a11de71e95f2154a4c9ce6d7` | `tflite_c_v2.17.1_darwin_arm64.tar.gz` |
| `x86_64-apple-darwin` | 2.17.0 | `6cf562771e6cb8d7856a86f859d004f0ef72861b6cafa88afbfad5d5f40261fc` | `tflite_c_v2.17.0_darwin_amd64.tar.gz` |

All assets come from the
[`tphakala/tflite_c`](https://github.com/tphakala/tflite_c/releases) releases.
TensorFlow is Apache-2.0 licensed; its license is included at
`LICENSES/tensorflow-LICENSE`.

Intel macOS uses 2.17.0 because that is the newest Intel artifact published by
this upstream distributor. The TensorFlow Lite C ABI used by
`micro-wakeword` is compatible across these patch releases.
