use serde::{Deserialize, Serialize};

#[derive(Serialize, Debug)]
pub struct ConfigurationRequestDTO {
    #[serde(rename(serialize = "macAddress"))]
    pub mac_address: String,
}

impl ConfigurationRequestDTO {
    pub fn new(mac_address: String) -> ConfigurationRequestDTO {
        ConfigurationRequestDTO { mac_address }
    }
}

#[derive(Deserialize, Debug)]
pub struct ConfigurationResponseDTO {
    #[serde(rename(deserialize = "powerOn"))]
    pub power_on: bool,
    #[serde(rename(deserialize = "macAddress"))]
    pub mac_address: String,
}
