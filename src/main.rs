//! Environment Monitoring application
use embedded_hal::i2c::I2c;
use environment_monitor_rust::bme68x_pure::{
    BME68xAddr, BME68xConf, BME68xDev, BME68xFilter, BME68xHeatrConf, BME68xIntf, BME68xODR,
    BME68xOpMode, BME68xOs,
};
use esp_idf_hal::delay::FreeRtos;
use esp_idf_hal::i2c::{I2cConfig, I2cDriver};
use esp_idf_hal::peripherals::Peripherals;
use esp_idf_hal::prelude::*;

/// Test `BME68x` Forced Mode
fn test_forced<I2C: I2c>(bme: &mut BME68xDev<I2C>) {
    bme.init().unwrap();
    let bme_conf = BME68xConf {
        filter: BME68xFilter::Off,
        os_hum: BME68xOs::Os16x,
        os_temp: BME68xOs::Os2x,
        os_pres: BME68xOs::Os1x,
        odr: BME68xODR::ODRNone,
    };

    bme.set_config(&bme_conf).unwrap();
    bme.set_heatr_conf_forced(300, 100).unwrap();

    for _ in 0..5 {
        bme.set_op_mode(BME68xOpMode::ForcedMode).unwrap();
        let del_period = bme.get_meas_dur(BME68xOpMode::ForcedMode, &bme_conf);
        FreeRtos::delay_us(del_period);
        let (data, _) = bme.get_data(BME68xOpMode::ForcedMode).unwrap();
        for (sample, entry) in data.iter().enumerate() {
            log::info!(
                "sample: {}, temp: {}, pres: {}, hum: {}, gas: {}, status: {}",
                sample,
                entry.temperature,
                entry.pressure,
                entry.humidity,
                entry.gas_resistance,
                entry.status,
            );
        }

        FreeRtos::delay_ms(1000);
    }
    log::info!("Finished Forced Test");
}

/// Test `BME68x` Parallel Mode
fn test_parallel<I2C: I2c>(bme: &mut BME68xDev<I2C>) {
    bme.init().unwrap();
    let bme_conf = BME68xConf {
        filter: BME68xFilter::Off,
        os_hum: BME68xOs::Os1x,
        os_temp: BME68xOs::Os2x,
        os_pres: BME68xOs::Os16x,
        odr: BME68xODR::ODRNone,
    };
    let bme_heater_conf = BME68xHeatrConf {
        enable: true,
        heatr_temp: 0,
        heatr_dur: 0,
        heatr_temp_prof: [320, 100, 100, 100, 200, 200, 200, 320, 320, 320],
        heatr_dur_prof: [5, 2, 10, 30, 5, 5, 5, 5, 5, 5],
        profile_len: 10,
        shared_heatr_dur: u16::try_from(
            140 - (bme.get_meas_dur(BME68xOpMode::ParallelMode, &bme_conf) / 1000),
        )
        .unwrap(),
    };
    bme.init().unwrap();
    bme.set_config(&bme_conf).unwrap();
    bme.set_heatr_conf_parallel(
        &bme_heater_conf.heatr_temp_prof,
        &bme_heater_conf.heatr_dur_prof,
    )
    .unwrap();

    bme.set_op_mode(BME68xOpMode::ParallelMode).unwrap();
    let mut sample_count = 0;
    while sample_count <= 50 {
        let del_period = bme.get_meas_dur(BME68xOpMode::ParallelMode, &bme_conf)
            + (u32::from(bme_heater_conf.shared_heatr_dur) * 1000);

        FreeRtos::delay_us(del_period);

        let read_result = bme.get_data(BME68xOpMode::ParallelMode);
        if read_result.is_err() {
            log::warn!("Sensor Error: {:?}", read_result);
        } else {
            let (data, n_fields) = read_result.unwrap();
            for entry in data.iter().take(n_fields as usize) {
                if entry.status == 0xB0 {
                    log::info!("sample: {}, temp: {}, pressure: {}, hum: {}, gas: {}, status: {}, gas_index: {}, meas_index: {}", sample_count, entry.temperature, entry.pressure, entry.humidity, entry.gas_resistance, entry.status, entry.gas_index, entry.meas_index);
                    sample_count += 1;
                }
            }
        }
    }
    log::info!("Finished parallel test");
}

/// Test `BME68x` Sequential Mode
fn test_sequential<I2C: I2c>(bme: &mut BME68xDev<I2C>) {
    bme.init().unwrap();
    let bme_conf = BME68xConf {
        filter: BME68xFilter::Off,
        os_hum: BME68xOs::Os16x,
        os_temp: BME68xOs::Os2x,
        os_pres: BME68xOs::Os1x,
        odr: BME68xODR::ODRNone,
    };
    let bme_heater_conf = BME68xHeatrConf {
        enable: true,
        heatr_temp: 0,
        heatr_dur: 0,
        heatr_temp_prof: [200, 240, 280, 320, 360, 360, 320, 280, 240, 200],
        heatr_dur_prof: [100, 100, 100, 100, 100, 100, 100, 100, 100, 100],
        profile_len: 10,
        shared_heatr_dur: 0,
    };
    bme.init().unwrap();
    bme.set_config(&bme_conf).unwrap();
    bme.set_heatr_conf(BME68xOpMode::SequentialMode, &bme_heater_conf)
        .unwrap();
    bme.set_op_mode(BME68xOpMode::SequentialMode).unwrap();

    let mut sample_count = 0;
    while sample_count <= 300 {
        let del_period = bme.get_meas_dur(BME68xOpMode::SequentialMode, &bme_conf)
            + (u32::from(bme_heater_conf.shared_heatr_dur) * 1000);
        FreeRtos::delay_us(del_period);

        let read_result = bme.get_data(BME68xOpMode::SequentialMode);
        if read_result.is_err() {
            log::warn!("Sensor Error: {:?}", read_result);
        } else {
            let (data, n_fields) = read_result.unwrap();
            for entry in data.iter().take(n_fields as usize) {
                if entry.status == 0xB0 {
                    log::info!("sample: {}, temp: {}, pressure: {}, hum: {}, gas: {}, status: {}, gas_index: {}, meas_index: {}", sample_count, entry.temperature, entry.pressure, entry.humidity, entry.gas_resistance, entry.status, entry.gas_index, entry.meas_index);
                    sample_count += 1;
                }
            }
        }
    }
    log::info!("Finished sequential test");
}

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

    let mut bme = BME68xDev::new(
        i2c_driver,
        BME68xAddr::HIGH,
        25,
        BME68xIntf::I2CIntf,
        Box::new(FreeRtos::delay_us),
    );

    bme.init().unwrap();
    bme.selftest_check().unwrap();
    test_forced(&mut bme);
    test_parallel(&mut bme);
    test_sequential(&mut bme);
}
