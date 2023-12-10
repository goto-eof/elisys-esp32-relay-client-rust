// WiFi SSID (WiFi name)
pub const WIFI_SSID: &str = "";
// WiFi password
pub const WIFI_PASS: &str = "";
// configuration server endpoint -> the endpoint from where ESP32 will download the configuration
pub const CONFIG_URI: &str = "http://localhost:8080/api/v1/relay/configuration";
// checks the configuration every X seconds
pub const CONFIG_CHECK_INTERVAL_SECONDS: u32 = 2;
// Device registration endpoint
pub const REGISTER_DEVICE_URL: &str = "http://192.168.1.102:8080/api/v1/device/register";
// Device name
pub const DEVICE_NAME: &str = "Relay";
// Device description
pub const DEVICE_DESCRIPTION: &str = "Relay Device";
// Device type
pub const DEVICE_TYPE: &str = "Relay";
