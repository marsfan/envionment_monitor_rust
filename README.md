# An ESP32-Based Environment Monitor in Rust.

This is a environment monitoring proram that reads from a few sensors, and
publishes the data with MQTT.

## Building

There are a couple of things needed to build.

1. When installing with `espup`, use the `--extended-llvm` flag.
2. Before using `cargo build`, source the `~/export_esp.sh` file that `esup` added to your home directory
