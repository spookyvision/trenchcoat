use config::{Config, File, FileFormat};
use serde::Deserialize;

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub(crate) struct AppConfig {
    pub(crate) pixel_count: usize,
    pub(crate) wifi_ssid: String,
    pub(crate) wifi_psk: String,
    pub(crate) data_pin: i32,
    pub(crate) clock_pin: Option<i32>,
}

impl AppConfig {
    pub(crate) fn new() -> Self {
        let config_data = File::from_str(include_str!("../config.toml"), FileFormat::Toml);
        let res: AppConfig = Config::builder()
            .add_source(config_data)
            .build()
            .unwrap()
            .try_deserialize()
            .unwrap();

        res
    }
}
