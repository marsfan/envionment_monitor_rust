//! Private config data that should not be committed to github

#[allow(clippy::doc_markdown)]
/// WiFi SSID
pub const SSID: &str = "ssid";

/// WiFi Password
pub const WIFI_PASS: &str = "pass";

/// MQTT URL
pub const AIO_MQTT_URL: &str = "mqtts://example.com:8883";

/// MQTT Username
pub const AIO_MQTT_USER: &str = "testaccount";

/// MQTT Password
pub const AIO_MQTT_PASS: &str = "1234567890";

// Adafruit IO MQTT Topics

/// Temperature topic
pub const AIO_TEMP_TOPIC: &str = "topics/dummy";

/// Pressure Topic
pub const AIO_PRES_TOPIC: &str = "topics/dummy";

/// Humidity Topic
pub const AIO_HUMIDITY_TOPIC: &str = "topics/dummy";

/// eCO2 Topic
pub const AIO_ECO2_TOPIC: &str = "topics/dummy";

/// IAQ Topic
pub const AIO_IAQ_TOPIC: &str = "topics/dummy";

/// Lux Topic
pub const AIO_LUX_TOPIC: &str = "topics/dummy";

/// TVOC Topic
pub const AIO_TVOC_TOPIC: &str = "topics/dummy";
