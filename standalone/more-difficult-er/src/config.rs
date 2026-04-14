use er_extend_rs_config::load_or_write_toml_config_file_alongside_dll;
use er_extend_rs_esd::config::ErExtendRsEsdConfig;
use serde::Deserialize;

#[derive(Debug, Deserialize, Default)]
pub struct MoreDifficultERConfig {
    pub log_debug_messages: Option<bool>,
    pub more_difficult_er_multiplier: Option<f32>,
    pub extra_config: ErExtendRsEsdConfig,
}

impl MoreDifficultERConfig {
    pub fn log_debug_messages(&self) -> bool {
        self.log_debug_messages.unwrap_or(false)
    }
}

pub fn get_config() -> MoreDifficultERConfig {
    let default_config = include_str!("../resources/more_difficult_er_config.toml");
    load_or_write_toml_config_file_alongside_dll("more_difficult_er_config.toml", "more_difficult_er.dll", default_config)
}