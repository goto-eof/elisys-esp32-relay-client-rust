# Elisys ESP32 Relay Client (Rust)

This application, developed in Rust programming language, allows to control remotely a relay controlled by an ESP32 device kit. The application will communicate with [**Elisys Home Automation (Java)**](https://github.com/goto-eof/elisys-home-automation-server-java) to retrieve the configuration and activate or deactivate the relay. Please check the `Configuration` section to build the project.

# How it works?

The application tries to connect to the WiFi, if the connection fails, then it will retry until it succeeds. After a connection is established, the application makes a request to the server in order to register the device and then it downloads the configuration from a remote server. The configuration contains information about the status that should have the device: on or off. So that the relay is disabled or enabled in base of the configuration JSON received by the remote server. The ESP32 application will check every 2 seconds for new configuration.

| GPIO | Description |
| ---- | ----------- |
| 15   | relay pin   |

# Configuration

Before you proceed with building the project, you need to rename the `/src/configuration/configuration-sample.rs` to `/src/configuration/configuration.rs`. Then you shall edit the variables in the `configuration.rs` file (WiFi SSID, WiFi Password and Configuration Server Endpoint)

# Run the project on your ESP32 device

```
cargo run
```

# Photo

![relay rust](/images/esp32-relay-client-rust.jpg)

Tested on ESP32-DevKitC and developed on Ubuntu.

If you found a bug, feel free to contact [me](https://andre-i.eu/#contactme).
