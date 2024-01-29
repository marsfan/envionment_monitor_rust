//! Implementation for sending data to MQTT brokers.
use esp_idf_hal::delay::FreeRtos;
use esp_idf_svc::mqtt::client::{EspMqttClient, MqttClientConfiguration, QoS};
use esp_idf_sys::esp_crt_bundle_attach;
use std::sync::{Arc, Mutex};

use crate::bsec::VirtualSensorData;
use crate::interconnect::SensorHubData;
use crate::private_data;
/// Task for sending data to a MQTT Broker
///
/// # Arguments
/// * `data_mutex`: The mutex for the sensor hub data
/// * `broker_url`: The MQTT Broker URL
/// * `username`: MQTT Broker Username
/// * `password`: MQTT Broker Password
/// * `sleep_time`: The time to sleep between each publish.
pub fn mqtt_task(
    data_mutex: &Arc<Mutex<SensorHubData>>,
    broker_url: &str,
    username: &str,
    password: &str,
    sleep_time: u32,
) {
    let mqtt_config = MqttClientConfiguration {
        crt_bundle_attach: Some(esp_crt_bundle_attach),
        username: Some(username),
        password: Some(password),
        ..Default::default()
    };

    let (mut client, mut connection) = EspMqttClient::new(broker_url, &mqtt_config).unwrap();

    // Need this for some reason to make the MQTT publishing working. Look at the esp-idf-svc mqtt client example
    // FIXME: Can I get rid of this?
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

        publish_bsec_data(
            &mut client,
            private_data::AIO_TEMP_TOPIC,
            data.bsec.compensated_temp,
            false,
        );

        publish_bsec_data(
            &mut client,
            private_data::AIO_PRES_TOPIC,
            data.bsec.raw_pressure,
            false,
        );

        publish_bsec_data(
            &mut client,
            private_data::AIO_HUMIDITY_TOPIC,
            data.bsec.compensated_humidity,
            false,
        );

        publish_bsec_data(
            &mut client,
            private_data::AIO_ECO2_TOPIC,
            data.bsec.co2_eq,
            false,
        );

        publish_bsec_data(
            &mut client,
            private_data::AIO_IAQ_TOPIC,
            data.bsec.iaq,
            false,
        );

        publish_bsec_data(
            &mut client,
            private_data::AIO_STATIC_IAQ,
            data.bsec.static_iaq,
            false,
        );

        publish_bsec_data(
            &mut client,
            private_data::AIO_TVOC_TOPIC,
            data.bsec.breath_voc_eq,
            false,
        );

        let payload = format!("{}", data.veml.lux);
        // FIXME: Log error instead of unwrapping
        client
            .publish(
                private_data::AIO_LUX_TOPIC,
                QoS::AtLeastOnce,
                false,
                payload.as_bytes(),
            )
            .unwrap();

        FreeRtos::delay_ms(sleep_time);
    }
}

/// Publish BSEC data to the given MQTT Client if the data is valid
///
/// # Arguments
/// * `client`: The MQTT client to publish to
/// * `topic`: The topic to publish to
/// * `data`: The data to publish.
/// * `metadata`: If true, publish a JSON that also includes the metadata.
///      If false, just publish the main data.
///
fn publish_bsec_data(
    client: &mut EspMqttClient,
    topic: &str,
    data: VirtualSensorData,
    metadata: bool,
) {
    if data.valid {
        // TODO: Publish if invalid but metadata = true?
        let payload = if metadata {
            // TODO: Use serde to create this.
            format!(
                "{{\"value\": {}, \"accuracy\": {}, \"timestamp\": {}}}",
                data.signal, data.accuracy, data.time_stamp
            )
        } else {
            format!("{}", data.signal)
        };

        // FIXME: Log error unstead of unwrap.
        client
            .publish(topic, QoS::AtLeastOnce, false, payload.as_bytes())
            .unwrap();
    } else {
        // TODO: Log that data was not valid?
    }
}
