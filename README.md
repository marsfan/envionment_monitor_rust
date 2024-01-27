# An ESP32-Based Environment Monitor in Rust.

This is a environment monitoring proram that reads from a few sensors, and
publishes the data with MQTT.

## Building

There are a few things needed to build.

1. When installing with `espup`, use the `--extended-llvm` flag.
2. Before using `cargo build`, source the `~/export_esp.sh` file that `esup` added to your home directory
3. The file `src/private_data.rs` which contains the following public constants.

|       Name        |  Type  |                  Purpose                   |
| ----------------- | ------ | ------------------------------------------ |
| `SSID`            | `&str` | WiFi SSID Name                             |
| `WIFI_PASS`       | `&str` | WiFi Password                              |
| `MQTT_URL`        | `&str` | MQTT Broker URL                            |
| `MQTT_USER`       | `&str` | MQTT Account Username                      |
| `MQTT_PASS`       | `&str` | MQTT Account password                      |
| `MQTT_TEMP_TOPIC` | `&str` | MQTT Topic for publishing the temperature. |

See the file [dummy_private_data.rs](src/dummy_private_data.rs) for an example