//! Environment Monitoring application
use environment_monitor_rust::bme68x_pure::{
    BME68xAddr, BME68xConf, BME68xData, BME68xDev, BME68xHeatrConf, BME68xIntf, BME68xODR,
    BME68xOpMode, BME68xOs,
};
use environment_monitor_rust::veml7700::Veml7700;
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

    // let mut veml = Veml7700::new(i2c_driver, 1000);
    let mut bme = BME68xDev::new(
        i2c_driver,
        BME68xAddr::HIGH,
        25,
        BME68xIntf::I2CIntf,
        Box::new(|delay| FreeRtos::delay_us(delay)),
    );
    let bme_conf = BME68xConf {
        filter: 0,
        os_hum: BME68xOs::Os16x,
        os_temp: BME68xOs::Os2x,
        os_pres: BME68xOs::Os1x,
        odr: BME68xODR::ODRNone,
    };
    let bme_heater_conf = BME68xHeatrConf {
        enable: 1,
        heatr_temp: 300,
        heatr_dur: 100,
        heatr_temp_prof: [0; 10],
        heatr_dur_prof: [0; 10],
        profile_len: 0,
        shared_heatr_dur: 0,
    };
    bme.init().unwrap();
    bme.set_config(&bme_conf).unwrap();
    bme.set_heatr_conf(BME68xOpMode::ForcedMode, &bme_heater_conf)
        .unwrap();
    loop {
        // let als = veml.get_ambient_level().unwrap();
        bme.set_op_mode(BME68xOpMode::ForcedMode).unwrap();
        let del_period = bme.get_meas_dur(BME68xOpMode::ForcedMode, &bme_conf);
        FreeRtos::delay_us(del_period as u32);
        let (data, status) = bme.get_data(BME68xOpMode::ForcedMode).unwrap();
        for (sample, entry) in data.iter().enumerate() {
            log::info!(
                "sample: {}, temp: {}, pres: {}, hum: {}, gas: {}, status: {}",
                sample,
                entry.temperature,
                entry.pressure,
                entry.humidity,
                entry.gas_resistance,
                entry.status,
            )
        }
        // log::info!("{als}");
        // bme.selftest_check().unwrap();
        FreeRtos::delay_ms(1000);
    }
}
