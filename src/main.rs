//! Environment Monitoring application

use environment_monitor_rust::veml7700::{Veml7700, VemlOutput};
use esp_idf_svc::eventloop::EspSystemEventLoop;
use esp_idf_svc::nvs::EspDefaultNvsPartition;
use esp_idf_svc::wifi::{BlockingWifi, ClientConfiguration, Configuration, EspWifi};
use std::sync::{mpsc, Arc, Mutex};
use std::thread;

use embedded_hal_bus::i2c::MutexDevice;
use environment_monitor_rust::bsec::bsec_bindings::BSEC_SAMPLE_RATE_LP;
use environment_monitor_rust::bsec::{Bsec, StructuredOutputs, VirtualSensorData};
use environment_monitor_rust::private_data::{SSID, WIFI_PASS};
use esp_idf_hal::delay::FreeRtos;
use esp_idf_hal::i2c::{I2cConfig, I2cDriver};
use esp_idf_hal::peripherals::Peripherals;
use esp_idf_hal::prelude::*;

/// Enumeration to hold data sent from sensor tasks to the sensor hub task.
#[derive(Debug)]
enum SensorData {
    /// Data from the BME688
    Bsec {
        /// The data from the sensor
        data: StructuredOutputs,
    },

    /// Data from the VEML7700 sensor.
    Veml {
        /// The data from the sensor
        data: VemlOutput,
    },
}

fn main() {
    // It is necessary to call this function once. Otherwise some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_svc::sys::link_patches();

    let peripherals = Peripherals::take().unwrap();

    // Set up I2C
    let i2c_config = I2cConfig::new().baudrate(100.kHz().into());
    let i2c_driver = I2cDriver::new(
        peripherals.i2c0,
        peripherals.pins.gpio32,
        peripherals.pins.gpio33,
        &i2c_config,
    )
    .unwrap();

    // System event loop
    let sys_loop = EspSystemEventLoop::take().unwrap();
    let nvs = EspDefaultNvsPartition::take().unwrap();

    // Set up WiFi
    let mut wifi_driver =
        EspWifi::new(peripherals.modem, sys_loop.clone(), Some(nvs.clone())).unwrap();
    let mut wifi = BlockingWifi::wrap(&mut wifi_driver, sys_loop.clone()).unwrap();
    wifi.set_configuration(&Configuration::Client(ClientConfiguration {
        ssid: SSID.try_into().unwrap(),
        password: WIFI_PASS.try_into().unwrap(),
        ..Default::default()
    }))
    .unwrap();
    wifi.start().unwrap();
    wifi.connect().unwrap();
    wifi.wait_netif_up().unwrap();

    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();

    // TODO: Was suggested also trying this.(declaring a `&'static Mutex`).
    // Seems I'm supposed to use lazy_static somehow.
    // Was told the following
    /* The Arc will Drop the inner type when you are
    done using it from all threads, but a &'static
    will stick around forever. This might be important
    if dropping the i2c driver disables the peripheral
    in order to reduce power usage. The embassy HALs do
    this for example, but I don't know if the esp hal does */
    // and
    /* A &'static T can be copied for free, but that doesn't
    matter too much since it is super cheap to clone an Arc */
    //
    // let i2c_mutex: &'static Mutex<I2cDriver> = &Mutex::new(i2c_driver);
    // let bsec_i2c = i2c_mutex.clone();
    // let veml_i2c = i2c_mutex.clone();
    let i2c_mutex = Arc::new(Mutex::new(i2c_driver));
    let bsec_i2c = i2c_mutex.clone();
    let veml_i2c = i2c_mutex.clone();

    // Set up channel for sensor tasks to send data over
    let (tx, rx) = mpsc::sync_channel(5);

    let bsec_transmitter = tx.clone();
    let veml_transmitter = tx.clone();

    // Set up mutex used to guard data in sensor hub
    let data_mutex = Arc::new(Mutex::new(SensorHubData::new()));
    let hub_data = data_mutex.clone();

    // Start the sensor hub thread
    thread::Builder::new()
        .name("Sensor Hub Thread".to_string())
        .stack_size(4096)
        .spawn(move || sensor_hub_task(&hub_data, &rx))
        .unwrap();

    // Start the sensor threads.
    thread::Builder::new()
        .name("BSEC Thread".to_string())
        .stack_size(4096)
        .spawn(move || bsec_task(&bsec_i2c, &bsec_transmitter))
        .unwrap();

    thread::Builder::new()
        .name("VEML Thread".to_string())
        .stack_size(4096)
        .spawn(move || veml_task(&veml_i2c, &veml_transmitter))
        .unwrap();

    // Main thread now handles periodically printing data read from the sensors
    loop {
        let sensor_hub_data = data_mutex.lock().unwrap();
        log_signal("Temp", sensor_hub_data.bsec.compensated_temp);
        log_signal("Humidity", sensor_hub_data.bsec.compensated_humidity);
        log_signal("Pressure", sensor_hub_data.bsec.raw_pressure);
        log_signal("Raw Gas", sensor_hub_data.bsec.raw_gas);
        log_signal("IAQ", sensor_hub_data.bsec.iaq);
        log_signal("Static IAQ", sensor_hub_data.bsec.static_iaq);
        log_signal("eCO2 IAQ", sensor_hub_data.bsec.co2_eq);
        log_signal("Breath VOC", sensor_hub_data.bsec.breath_voc_eq);
        log_signal("Gas Percent", sensor_hub_data.bsec.gas_percentage);
        log_signal("Run In Status", sensor_hub_data.bsec.run_in_status);
        log_signal("Stabilization", sensor_hub_data.bsec.stabilization_status);
        log::info!(
            "ALS: {}, White: {}, Lux: {}",
            sensor_hub_data.veml.raw_als,
            sensor_hub_data.veml.raw_white,
            sensor_hub_data.veml.lux,
        );
        log::info!("----------------------------------------");

        FreeRtos::delay_ms(2000);
    }
}

/// Task for processing data from the BME688 with BSEC
///
/// # Arguments
/// * `i2c_handle`: Handle to a Mutex-protected I2C driver used to
///     communicate with the sensor.
/// * `transmitter`: The transmitter that will be used to send data to the sensor hub thread
fn bsec_task(i2c_handle: &Arc<Mutex<I2cDriver<'_>>>, transmitter: &mpsc::SyncSender<SensorData>) {
    let i2c_driver = MutexDevice::new(i2c_handle);
    let mut bsec = Bsec::new(i2c_driver, 0.0);

    log::info!("Starting BSEC");
    bsec.init().unwrap();
    bsec.subscribe_all_non_scan(BSEC_SAMPLE_RATE_LP as f32)
        .unwrap();
    let version = bsec.get_version().unwrap();
    log::info!(
        "BSEC Version: {}.{}.{}.{}",
        version.major,
        version.minor,
        version.major_bugfix,
        version.minor_bugfix
    );

    loop {
        // FIXME: Find safe alternative
        bsec.periodic_process(unsafe { esp_idf_sys::esp_timer_get_time() } * 1000)
            .unwrap();

        let data = bsec.get_output_data();

        transmitter.send(SensorData::Bsec { data }).unwrap();

        let remaining_time =
            bsec.get_next_call_time_us() - unsafe { esp_idf_sys::esp_timer_get_time() };

        FreeRtos::delay_us(remaining_time.try_into().unwrap());
    }
}

/// Task for reading data from the VEML7700 sensor.
///
/// # Arguments
/// * `i2c_handle`: Handle to a Mutex-protected I2C driver used to
///     communicate with the sensor.
/// * `transmitter`: The transmitter that will be used to send data to the sensor hub thread.
fn veml_task(i2c_handle: &Arc<Mutex<I2cDriver<'_>>>, transmitter: &mpsc::SyncSender<SensorData>) {
    let i2c_driver = MutexDevice::new(i2c_handle);
    let mut veml = Veml7700::new(i2c_driver, 1000);
    veml.set_power_state(false).unwrap();

    loop {
        veml.periodic_process();
        let data = veml.get_outputs();
        transmitter.send(SensorData::Veml { data }).unwrap();

        FreeRtos::delay_ms(1000);
    }
}

/// Task for the sensor hub
///
/// # Arguments
/// * `receiver`: The receiver that will get data from the sensor tasks.
/// * `data_mutex`: Mutex protected sensor data that the sensor hub will collect.
fn sensor_hub_task(data_mutex: &Arc<Mutex<SensorHubData>>, receiver: &mpsc::Receiver<SensorData>) {
    loop {
        // Read here first so that we don't try to acquire the mutex until we have
        // data to act on
        let received_data = receiver.recv().unwrap();
        // Lock mutex so we can safely work with the data.
        let mut locked_mutex = data_mutex.lock().unwrap();
        // Copy over the most recently send data from the channel into the structure.
        match received_data {
            SensorData::Bsec { data } => locked_mutex.bsec = data,
            SensorData::Veml { data } => locked_mutex.veml = data,
        }
    }
}

/// Log BSEC signals to the console
///
/// # Arguments
/// * `name`: The name of the signal to log
/// * `signal`: The signal to log
fn log_signal(name: &str, value: VirtualSensorData) {
    log::info!(
        "{name}: {}, Acc: {}, Valid: {}",
        value.signal,
        value.accuracy,
        value.valid,
    );
}

/// Structure for holding data from all of the sensors
#[derive(Clone, Copy)]
struct SensorHubData {
    /// Data from the BME688 sensor
    pub bsec: StructuredOutputs,

    /// Data from the VEML7700 sensor
    pub veml: VemlOutput,
}

impl SensorHubData {
    /// Create a new instance of the structure
    pub fn new() -> Self {
        Self {
            bsec: StructuredOutputs::new(),
            veml: VemlOutput::new(),
        }
    }
}
