use super::{
    client_service::retrieve_configuration,
    wifi_service::{connect_reconnect_wifi_if_necessary, get_mac_address},
};
use crate::{
    configuration::configuration::{CONFIG_CHECK_INTERVAL_SECONDS, CONFIG_URI},
    helper::orchestrator_helper::{process_configuration, try_register_device},
};
use esp_idf_hal::{delay::FreeRtos, gpio::PinDriver, peripherals::Peripherals};
use esp_idf_svc::{eventloop::EspSystemEventLoop, nvs::EspDefaultNvsPartition, wifi::EspWifi};
use log::error;

pub fn orchestrate() -> anyhow::Result<()> {
    let peripherals = Peripherals::take().unwrap();
    let mut relay = PinDriver::input_output(peripherals.pins.gpio15)?;

    let sys_loop = EspSystemEventLoop::take().unwrap();
    let nvs = EspDefaultNvsPartition::take().unwrap();
    let mut wifi_driver = EspWifi::new(peripherals.modem, sys_loop, Some(nvs)).unwrap();

    connect_reconnect_wifi_if_necessary(&mut wifi_driver);

    let mac_address = get_mac_address(&mut wifi_driver);

    FreeRtos::delay_ms(2000); // need to wait some time because the first request fails (perhaps wifi is not completely ready?)

    try_register_device(&mac_address);

    loop {
        connect_reconnect_wifi_if_necessary(&mut wifi_driver);

        match retrieve_configuration(CONFIG_URI, &mac_address) {
            Err(e) => error!("Something went wrong: {:?}", e),
            Ok(configuration_result) => process_configuration(configuration_result, &mut relay),
        };

        FreeRtos::delay_ms(CONFIG_CHECK_INTERVAL_SECONDS * 1000);
    }
}
