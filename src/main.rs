use crate::configuration::configuration::{
    CONFIG_CHECK_INTERVAL_SECONDS, CONFIG_URI, DEVICE_DESCRIPTION, DEVICE_NAME, DEVICE_TYPE,
    REGISTER_DEVICE_URL, WIFI_PASS, WIFI_SSID,
};
use crate::dto::configuration_dto::ConfigurationRequestDTO;
use crate::dto::register_device::RegisterDeviceDTO;
use anyhow::{self, Error};
use dto::configuration_dto::ConfigurationResponseDTO;
use embedded_svc::{http::client::Client as HttpClient, io::Write, utils::io};
use esp_idf_hal::delay::FreeRtos;
use esp_idf_hal::gpio::*;
use esp_idf_hal::peripherals::Peripherals;
use esp_idf_svc::eventloop::EspSystemEventLoop;
use esp_idf_svc::http::client::EspHttpConnection;
use esp_idf_svc::nvs::EspDefaultNvsPartition;
use esp_idf_svc::wifi::{
    ClientConfiguration, Configuration as WifiConfiguration, EspWifi, WifiDeviceId,
};
use esp_idf_sys::EspError;
use log::{error, info, warn};
use std::result::Result::Ok as StandardOk;
pub mod configuration;
pub mod dto;
fn main() -> anyhow::Result<()> {
    esp_idf_sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals = Peripherals::take().unwrap();
    let mut relay = PinDriver::input_output(peripherals.pins.gpio15)?;

    let sys_loop = EspSystemEventLoop::take().unwrap();
    let nvs = EspDefaultNvsPartition::take().unwrap();

    let mut wifi_driver = EspWifi::new(peripherals.modem, sys_loop, Some(nvs)).unwrap();
    info!("trying to connect to the wifi network....");
    connect_reconnect_wifi_if_necessary(&mut wifi_driver);
    info!("connection done");
    let mac_address = get_mac_address(&mut wifi_driver);
    info!("mac address: {}", mac_address);
    FreeRtos::delay_ms(2000);
    let register_device_result = register_device(&mac_address);
    if register_device_result.is_err() {
        error!(
            "Failed to register the device: {:?}",
            register_device_result
        );
    } else {
        info!("Device registered successfully!");
    }
    loop {
        info!("trying to connect to the wifi network if necessary....");
        connect_reconnect_wifi_if_necessary(&mut wifi_driver);
        info!("connection done");
        info!("trying to retrieve configuration...");
        let configuration_result = retrieve_configuration(CONFIG_URI, &mac_address);
        info!("done");

        if configuration_result.is_ok() {
            let configuration = configuration_result.unwrap();
            if configuration.power_on && relay.is_low() {
                if relay.set_high().is_err() {
                    warn!("unable to activate the device");
                } else {
                    info!("device activated");
                }
            }
            if !configuration.power_on && relay.is_high() {
                if relay.set_low().is_err() {
                    warn!("unable to deactivate the device");
                } else {
                    info!("device deactivated");
                }
            }
        } else {
            error!("error: {:?}", configuration_result.err());
        }
        FreeRtos::delay_ms(CONFIG_CHECK_INTERVAL_SECONDS * 1000);
    }
}

pub fn get_mac_address(wifi: &mut EspWifi<'static>) -> String {
    let mav = wifi.driver().get_mac(WifiDeviceId::Sta).unwrap();
    let mac_address_obj = macaddr::MacAddr6::new(mav[0], mav[1], mav[2], mav[3], mav[4], mav[5]);
    let mac_address_value = mac_address_obj.to_string();
    info!("eifi: MAC_ADDRESS: {:?}", mac_address_value);
    mac_address_value
}

fn connect_reconnect_wifi_if_necessary(wifi_driver: &mut EspWifi<'static>) {
    while wifi_driver.is_connected().is_err() || !wifi_driver.is_connected().unwrap() {
        info!("trying to connect to wifi....");
        if connect_wifi(wifi_driver).is_err() {
            FreeRtos::delay_ms(500);
            info!("retrying wifi");
        } else {
            info!("connected to wifi :)");
            break;
        }
    }
}

fn connect_wifi<'a>(
    wifi_driver: &'a mut EspWifi<'static>,
) -> Result<&'a mut EspWifi<'static>, EspError> {
    info!("wifi: setting configuration...");
    wifi_driver.set_configuration(&WifiConfiguration::Client(ClientConfiguration {
        ssid: WIFI_SSID.into(),
        password: WIFI_PASS.into(),
        ..Default::default()
    }))?;
    info!("wifi: starting device...");
    wifi_driver.start()?;
    info!("wifi: connecting...");
    wifi_driver.connect()?;
    info!("wifi: connected: {:?}", wifi_driver.is_connected());

    while !wifi_driver.is_connected()? {
        let config = wifi_driver.get_configuration()?;
        warn!("wifi: waiting for connection establishment: {:?}", config);
        FreeRtos::delay_ms(1000);
    }
    info!("wifi: connected!");
    return Ok(wifi_driver);
}

pub fn retrieve_configuration(
    configuration_uri: &str,
    mac_address: &str,
) -> anyhow::Result<ConfigurationResponseDTO, anyhow::Error> {
    info!("connecting to: {}", configuration_uri);
    let client = HttpClient::wrap(EspHttpConnection::new(&Default::default())?);
    let payload = serde_json::to_string(&ConfigurationRequestDTO::new(mac_address.into())).unwrap();
    let payload = payload.as_bytes();

    info!("trying retrieve configuration...");
    let result = post_request(payload, client, configuration_uri);
    info!("configuration retrieved? {}", !result.is_err());
    match result {
        StandardOk(body_string) => {
            let configuration: Result<ConfigurationResponseDTO, serde_json::Error> =
                serde_json::from_str(&body_string);
            info!("{:?}", configuration);

            if configuration.is_err() {
                let err = configuration.err().unwrap();
                error!(
            "[config downloader]: error while trying to parse the configuration response: {}",
            &err
        );
                return Err(err.into());
            }

            let configuration = configuration.unwrap();
            info!(
                "[config downloader]: Remote configuration loaded successfully: {:?}",
                configuration
            );
            return Ok(configuration);
        }
        Err(e) => {
            error!("[config downloader]: Error decoding response body: {}", e);
            return Err(e.into());
        }
    }
}

fn post_request(
    payload: &[u8],
    mut client: HttpClient<EspHttpConnection>,
    url: &str,
) -> Result<String, Error> {
    let content_length_header = format!("{}", payload.len());
    let headers = [
        ("content-type", "application/json"),
        ("content-length", &*content_length_header),
    ];

    let request = client.post(url, &headers);

    if request.is_err() {
        let message = format!("connection error: {:?}", request.err());
        error!("{}", message);
        return Err(Error::msg(message));
    }
    let mut request = request.unwrap();

    if request.write_all(payload).is_err() {
        let message = format!("connection error while trying to write all");
        error!("{}", message);
        return Err(Error::msg(message));
    }
    if request.flush().is_err() {
        let message = format!("connection error while trying to flush");
        error!("{}", message);
        return Err(Error::msg(message));
    }
    info!("-> POST {}", url);
    let response = request.submit();
    if response.is_err() {
        let message = format!("connection error while trying to read response");
        error!("{}", message);
        return Err(Error::msg(message));
    }
    let mut response = response.unwrap();

    let status = response.status();
    info!("<- {}", status);
    let mut buf = [0u8; 4086];
    let bytes_read = io::try_read_full(&mut response, &mut buf).map_err(|e| e.0);

    if bytes_read.is_err() {
        let message = format!(
            "connection error while trying to read response: {:?}",
            bytes_read.err()
        );
        error!("{}", message);
        return Err(Error::msg(message));
    } else {
        let bytes_read = bytes_read.unwrap();
        return match std::str::from_utf8(&buf[0..bytes_read]) {
            Err(e) => Err(Error::msg(format!("{:?}", e))),
            StandardOk(str) => Ok(str.to_owned()),
        };
    }
}

pub fn register_device(mac_address: &str) -> anyhow::Result<(), anyhow::Error> {
    let client = HttpClient::wrap(EspHttpConnection::new(&Default::default())?);

    let payload = serde_json::to_string(&RegisterDeviceDTO::new(
        mac_address.to_owned(),
        DEVICE_TYPE.into(),
        DEVICE_NAME.into(),
        DEVICE_DESCRIPTION.into(),
    ))
    .unwrap();
    let payload = payload.as_bytes();

    info!("trying to send data...");
    let result = post_request(payload, client, REGISTER_DEVICE_URL);
    info!("data sent? {}", !result.is_err());
    return match result {
        Err(e) => Err(e.into()),
        StandardOk(_) => Ok(()),
    };
}
