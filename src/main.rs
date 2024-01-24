//! Environment Monitoring application

use environment_monitor_rust::veml7700::Veml7700;
use std::sync::{Arc, Mutex};
use std::thread;

use embedded_hal_bus::i2c::MutexDevice;
use environment_monitor_rust::bsec::bsec_datatypes_bindings::BSEC_SAMPLE_RATE_LP;
use environment_monitor_rust::bsec::{Bsec, BsecVirtualSensorData};
use esp_idf_hal::delay::FreeRtos;
use esp_idf_hal::i2c::{I2cConfig, I2cDriver};
use esp_idf_hal::peripherals::Peripherals;
use esp_idf_hal::prelude::*;

fn main() {
    // It is necessary to call this function once. Otherwise some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_svc::sys::link_patches();

    let peripherals = Peripherals::take().unwrap();

    let i2c_config = I2cConfig::new().baudrate(100.kHz().into());
    let i2c_driver = I2cDriver::new(
        peripherals.i2c0,
        peripherals.pins.gpio32,
        peripherals.pins.gpio33,
        &i2c_config,
    )
    .unwrap();

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

    // Start the sensor threads.
    thread::Builder::new()
        .name("BSEC Thread".to_string())
        .stack_size(4096)
        .spawn(move || bsec_task(bsec_i2c))
        .unwrap();

    thread::Builder::new()
        .name("VEML Thread".to_string())
        .stack_size(2048)
        .spawn(move || veml_task(veml_i2c))
        .unwrap();
}

/// Task for processing data from the BME688 with BSEC
///
/// # Arguments
/// * `i2c_handle`: Handle to a Mutex-protected I2C driver used to
///     communicate with the sensor.
fn bsec_task(i2c_handle: Arc<Mutex<I2cDriver<'_>>>) {
    let i2c_driver = MutexDevice::new(&i2c_handle);
    let mut bsec = Bsec::new(i2c_driver, 25.0);

    log::info!("Starting BSEC");
    bsec.init().unwrap();
    bsec.subscribe_all_non_scan(BSEC_SAMPLE_RATE_LP).unwrap();
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

        // Log the data
        let data = bsec.get_output_data();
        log_signal("Temp", data.compensated_temp);
        log_signal("Humidity", data.compensated_humidity);
        log_signal("Pressure", data.raw_pressure);
        log_signal("Raw Gas", data.raw_gas);
        log_signal("IAQ", data.iaq);
        log_signal("Static IAQ", data.static_iaq);
        log_signal("eCO2 IAQ", data.co2_eq);
        log_signal("Breath VOC", data.breath_voc_eq);
        log_signal("Gas Percent", data.gas_percentage);
        log_signal("Run In Status", data.run_in_status);
        log_signal("Stabilization", data.stabilization_status);
        log_signal("Raw Temp", data.raw_temp);
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
fn veml_task(i2c_handle: Arc<Mutex<I2cDriver<'_>>>) {
    let i2c_driver = MutexDevice::new(&i2c_handle);
    let mut veml = Veml7700::new(i2c_driver, 1000);
    veml.set_power_state(false).unwrap();

    loop {
        veml.periodic_process();
        log::info!("LUX: {}", veml.get_lux().unwrap());
        let veml_data = veml.get_outputs();
        log::info!(
            "Veml ALS: {}, White: {}, lux: {}",
            veml_data.raw_als,
            veml_data.raw_white,
            veml_data.lux
        );

        FreeRtos::delay_ms(1000);
    }
}

/// Log BSEC signals to the console
///
/// # Arguments
/// * `name`: The name of the signal to log
/// * `signal`: The signal to log
fn log_signal(name: &str, value: BsecVirtualSensorData) {
    log::info!(
        "{name}: {}, Acc: {}, Valid: {}",
        value.signal,
        value.accuracy,
        value.valid,
    );
}
