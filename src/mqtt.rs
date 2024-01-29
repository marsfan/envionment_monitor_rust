//! Implementation for sending data to MQTT brokers.
use esp_idf_hal::delay::FreeRtos;
use esp_idf_svc::mqtt::client::{EspMqttClient, MqttClientConfiguration, QoS};
use esp_idf_sys::esp_crt_bundle_attach;
use std::sync::{Arc, Mutex};

use crate::interconnect::SensorHubData;
use crate::private_data;
/// Task for sending data to a MQTT Broker
///
/// # Arguments
/// * `data_mutex`: The mutex for the sensor hub data
/// * `broker_url`: The MQTT Broker URL
/// * `username`: MQTT Broker Username
/// * `password`: MQTT Broker Password
pub fn mqtt_task(
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

    // Need this for some reason to make the MQTT publishing working. Look at the esp-idf-svc mqtt client example
    std::thread::Builder::new()
        .stack_size(6000)
        .spawn(move || {
            log::info!("MQTT Listening for messages");

            while let Ok(event) = connection.next() {
                log::info!("[Queue] Event: {}", event.payload());
            }

            log::info!("Connection closed");
        })
        .unwrap();

    loop {
        // Get The data and release the mutex as quickly as possible.

        let locked_mutex = data_mutex.lock().unwrap();
        let data = *locked_mutex;
        drop(locked_mutex);

        let payload = format!("{}", data.bsec.compensated_temp.signal);

        client
            .publish(
                private_data::AIO_TEMP_TOPIC,
                QoS::AtMostOnce,
                false,
                payload.as_bytes(),
            )
            .unwrap();

        FreeRtos::delay_ms(20000);
    }
}
