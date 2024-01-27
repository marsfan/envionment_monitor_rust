//! Private config data that should not be committed to github

#[allow(clippy::doc_markdown)]
/// WiFi SSID
pub const SSID: &str = "ssid";

/// WiFi Password
pub const WIFI_PASS: &str = "pass";

/// MQTT URL
pub const MQTT_URL: &str = "mqtts://example.com:8883";

/// MQTT Username
pub const MQTT_USER: &str = "testaccount";

/// MQTT Password
pub const MQTT_PASS: &str = "1234567890";

/// MQTT Temperature topic
pub const MQTT_TEMP_TOPIC: &str = "data/test-feed";
