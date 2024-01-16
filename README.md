# An ESP32-Based Environment Monitor in Rust.

This is a environment monitoring proram that reads from a few sensors, and
publishes the data with MQTT.

## Building

There are a couple of things needed to build.

1. When installing with `espup`, use the `--extended-llvm` flag.
2. Before using `cargo build`, source the `~/export_esp.sh` file that `esup` added to your home directory

In build.rs, the logic that was ussed to generate the bindings is commented out.
If for any reason you need to re-generate the bindings, uncomment that logic.
I did this for two reasons.

1. Bindgen was being called on every invocation, which I did not want.
2. It allowed me to modify the bindings afterwards and tweak tweak them as needed.