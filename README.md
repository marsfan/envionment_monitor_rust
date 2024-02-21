# An ESP32-Based Environment Monitor in Rust.

This is a environment monitoring proram that reads from a few sensors, and
publishes the data with MQTT.

## Build Requirements

To build the project, you will need the ESP32 rust toolchain. This can be
be installed using the `espup` utility, which is installed with `cargo install espup`.

Install the toolchain with `espup install --extended-llvm`

Once the toolchain is installed, all cargo commands should be run with
`cargo +esp COMMAND`, or you should set a directory override for the
project directory using `rustup override set esp` on the project directory.

## Building

There are a few changes that must be done to the codebase before it can be
successfully built.

### Add the BSEC library

The Bosch BSEC library is not released under an open-source license.
Download it from <https://www.bosch-sensortec.com/software-tools/software/bme680-software-bsec/>
and add the `inc` and `bin` folders to the
[environment-monitor/src/bsec](/environment-monitor/src/bsec) directory


### Confirgure Private Data

There are a few configuration options that will be unique for each user.
They are all set in the file
[environment-monitor/src/private_data.rs](environment-monitor/src/private_data.rs)

They are as follows:

|          Name          |  Type  |                         Purpose                          |
| ---------------------- | ------ | -------------------------------------------------------- |
| `SSID`                 | `&str` | WiFi SSID Name                                           |
| `WIFI_PASS`            | `&str` | WiFi Password                                            |
| `AIO_MQTT_URL`         | `&str` | MQTT Broker URL for Adafruit IO                          |
| `AIO_MQTT_USER`        | `&str` | MQTT Account Username for Adafruit IO                    |
| `AIO_MQTT_PASS`        | `&str` | MQTT Account password for Adafruit IO                    |
| `AIO_TEMP_TOPIC`       | `&str` | MQTT Topic for publishing the temperature to Adafruit IO |
| `AIO_PRES_TOPIC`       | `&str` | MQTT Topic for publishing the pressure to Adafruit IO    |
| `AIO_HUMIDITY_TOPIC`   | `&str` | MQTT Topic for publishing the humidity to Adafruit IO    |
| `AIO_ECO2_TOPIC`       | `&str` | MQTT Topic for publishing the eCO2 to Adafruit IO        |
| `AIO_IAQ_TOPIC`        | `&str` | MQTT Topic for publishing the IAQ to Adafruit IO         |
| `AIO_STATIC_IAQ_TOPIC` | `&str` | MQTT Topic for publishing static IAQ to Adafruit IO      |
| `AIO_TVOC_TOPIC`       | `&str` | MQTT Topic for publishing the TVOC to Adafruit IO        |
| `AIO_LUX_TOPIC`        | `&str` | MQTT Topic for publishing the Lux to Adafruit IO         |

See the file [dummy_private_data.rs](src/dummy_private_data.rs) for an example


## Running Unit Tests

Tests can be run either on hardware, or via QEMU. Both ways currently require
changing the runner away from the default, so I have provided scripts that
perform the necessary changes.

* For running unit tests on hardware, execute the file [run_tests_hw.sh](run_tests_hw.sh)
* For running unit tests via QEMU, execute the file [test_wrapper.py](test_wrapper.py)
  * This script requires Python 3 installed, and the `pexpect` python module.

Neither of these scripts require any arguments.

Due to limitations in how `cargo test` works on the ESP32, the entire test
binary will crash, only one failing test can be identified per run.
Fiding a fix for this is WIP.

When running via QEMU, the script will automatically kill the running
session if it detects either a failed test binary, or a full success.
The script for running test on HW does not current support this. That is
currently WIP.

