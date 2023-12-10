use esp_idf_hal::delay::FreeRtos;
use esp_idf_svc::wifi::{
    ClientConfiguration, Configuration as WifiConfiguration, EspWifi, WifiDeviceId,
};
use esp_idf_sys::EspError;
use log::{info, warn};

use crate::configuration::configuration::{WIFI_PASS, WIFI_SSID};
pub fn get_mac_address(wifi: &mut EspWifi<'static>) -> String {
    let mav = wifi.driver().get_mac(WifiDeviceId::Sta).unwrap();
    let mac_address_obj = macaddr::MacAddr6::new(mav[0], mav[1], mav[2], mav[3], mav[4], mav[5]);
    let mac_address_value = mac_address_obj.to_string();
    info!("eifi: MAC_ADDRESS: {:?}", mac_address_value);
    mac_address_value
}

pub fn connect_reconnect_wifi_if_necessary(wifi_driver: &mut EspWifi<'static>) {
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

pub fn connect_wifi<'a>(
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
