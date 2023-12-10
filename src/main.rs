pub mod configuration;
pub mod dto;
pub mod helper;
pub mod service;

use anyhow::{self};
use service::orchestrator_service::orchestrate;

fn main() -> anyhow::Result<()> {
    esp_idf_sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    return orchestrate();
}
