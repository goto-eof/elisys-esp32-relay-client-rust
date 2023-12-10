use esp_idf_hal::gpio::{Gpio15, InputOutput, PinDriver};
use log::{error, info, warn};

use crate::service::client_service::register_device;

pub fn process_configuration(
    configuration: crate::dto::configuration_dto::ConfigurationResponseDTO,
    relay: &mut PinDriver<'_, Gpio15, InputOutput>,
) {
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
}

pub fn try_register_device(mac_address: &String) {
    let register_device_result = register_device(mac_address);

    if register_device_result.is_err() {
        error!(
            "Failed to register the device: {:?}",
            register_device_result
        );
    } else {
        info!("Device registered successfully!");
    }
}
