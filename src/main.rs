//! Environment Monitoring application

use environment_monitor_rust::veml7700::{Veml7700, VemlOutput};
use esp_idf_hal::cpu::Core;
use esp_idf_hal::task::thread::ThreadSpawnConfiguration;
use esp_idf_svc::eventloop::EspSystemEventLoop;
use esp_idf_svc::mqtt::client::{EspMqttClient, MqttClientConfiguration, QoS};
use esp_idf_svc::nvs::EspDefaultNvsPartition;
use esp_idf_svc::wifi::{BlockingWifi, ClientConfiguration, Configuration, EspWifi};
use esp_idf_sys::{esp_crt_bundle_attach, esp_mqtt_client_config_t_credentials_t_authentication_t};
use std::io;
use std::sync::{mpsc, Arc, Mutex};
use std::thread::{self, JoinHandle};

use embedded_hal_bus::i2c::MutexDevice;
use environment_monitor_rust::bsec;
use environment_monitor_rust::private_data::{
    MQTT_PASS, MQTT_TEMP_TOPIC, MQTT_URL, MQTT_USER, SSID, WIFI_PASS,
};
use esp_idf_hal::delay::FreeRtos;
use esp_idf_hal::i2c::{I2cConfig, I2cDriver};
use esp_idf_hal::peripherals::Peripherals;
use esp_idf_hal::prelude::*;

/// Enumeration to hold data sent from sensor tasks to the sensor hub task.
#[derive(Debug)]
#[allow(clippy::large_enum_variant)]
enum SensorData {
    /// Data from the BME688
    Bsec {
        /// The data from the sensor
        data: bsec::StructuredOutputs,
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
    let adafruit_io_data = data_mutex.clone();

    spawn_thread(b"Sensor Hub Thread\0", 4096, 2, None, move || {
        sensor_hub_task(&hub_data, &rx);
    })
    .unwrap();

    spawn_thread(b"BSEC Thread\0", 4096, 1, None, move || {
        bsec_task(&bsec_i2c, &bsec_transmitter);
    })
    .unwrap();

    spawn_thread(b"VEML Thread\0", 4096, 1, None, move || {
        veml_task(&veml_i2c, &veml_transmitter);
    })
    .unwrap();

    spawn_thread(b"Adafruit IO Thread\0", 4096, 1, None, move || {
        mqtt_task(&adafruit_io_data, MQTT_URL, MQTT_USER, MQTT_PASS);
    })
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
    let mut bsec = bsec::Bsec::new(i2c_driver, 0.0);

    log::info!("Starting BSEC");
    bsec.init().unwrap();
    bsec.subscribe_all_non_scan(bsec::SampleRate::LowPower)
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

        let remaining_time_32 = u32::try_from(remaining_time);
        if let Ok(remaining_time_32) = remaining_time_32 {
            FreeRtos::delay_ms(remaining_time_32 * 1000);
        } else {
            log::warn!("Bad Remaining Time: {remaining_time}");
            FreeRtos::delay_ms(1000);
        }
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

/// Task for sending data to a MQTT Broker
///
/// # Arguments
/// * `data_mutex`: The mutex for the sensor hub data
/// * `broker_url`: The MQTT Broker URL
/// * `username`: MQTT Broker Username
/// * `password`: MQTT Broker Password
fn mqtt_task(
    data_mutex: &Arc<Mutex<SensorHubData>>,
    broker_url: &str,
    username: &str,
    password: &str,
) {
    let mqtt_config = MqttClientConfiguration {
        crt_bundle_attach: Some(esp_crt_bundle_attach),
        username: Some(username),
        password: Some(password),
        ..Default::default()
    };

    let (mut client, mut connection) = EspMqttClient::new(broker_url, &mqtt_config).unwrap();

    loop {
        let locked_mutex = data_mutex.lock().unwrap();
        let payload = format!("{}", locked_mutex.bsec.compensated_temp.signal);

        // FIXME: THis is not working
        client
            .publish(MQTT_TEMP_TOPIC, QoS::AtMostOnce, false, payload.as_bytes())
            .unwrap();

        FreeRtos::delay_ms(6000);
    }
}

/// Log BSEC signals to the console
///
/// # Arguments
/// * `name`: The name of the signal to log
/// * `signal`: The signal to log
fn log_signal(name: &str, value: bsec::VirtualSensorData) {
    log::info!(
        "{name}: {}, Acc: {}, Valid: {}",
        value.signal,
        value.accuracy,
        value.valid,
    );
}

/// Spawn a thread with extra ESP-32 specific options
///
/// # Arguments
/// * `name`: The name to give the thread
/// * `stack_size`: The size of the stack to give the thread.
/// * `priority`: The task priority. Higher number = higher prioritty
/// * `pinning`: What core to pin the task to (if any)
/// * `task`: The actual task
// FIXME: How can we pass in a slice and auto-append null there?
fn spawn_thread<F, T>(
    name: &'static [u8],
    stack_size: usize,
    priority: u8,
    pinning: Option<Core>,
    task: F,
) -> io::Result<JoinHandle<T>>
where
    F: FnOnce() -> T,
    F: Send + 'static,
    T: Send + 'static,
{
    ThreadSpawnConfiguration {
        name: Some(name),
        stack_size,
        priority,
        pin_to_core: pinning,
        ..Default::default()
    }
    .set()
    .unwrap();

    let thread_handle = thread::Builder::new()
        .name(
            String::from_utf8(Vec::from(name))
                .unwrap()
                .replace('\x00', ""),
        )
        .stack_size(stack_size)
        .spawn(task);

    ThreadSpawnConfiguration::default().set().unwrap();

    thread_handle
}

/// Structure for holding data from all of the sensors
#[derive(Clone, Copy)]
struct SensorHubData {
    /// Data from the BME688 sensor
    pub bsec: bsec::StructuredOutputs,

    /// Data from the VEML7700 sensor
    pub veml: VemlOutput,
}

impl SensorHubData {
    /// Create a new instance of the structure
    pub fn new() -> Self {
        Self {
            bsec: bsec::StructuredOutputs::new(),
            veml: VemlOutput::new(),
        }
    }
}
