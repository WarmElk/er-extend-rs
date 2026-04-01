use er_extend_rs_config::load_or_write_toml_config_file_alongside_dll;
use er_extend_rs_esd::config::ErExtendRsEsdConfig;
use serde::Deserialize;

const BASE_MOD_ID: u32 = 1061470000;
const BASE_TEMPORARY_FLAG_ID: u32 = BASE_MOD_ID + 5000;
const BASE_SAVED_FLAG_ID: u32 = BASE_MOD_ID;

pub const UPGRADE_ALL_WEAPONS_FLAG_ID: u32 = BASE_TEMPORARY_FLAG_ID + 1;
pub const TOGGLE_UPGRADE_STATS_DISPLAY_FLAG_ID: u32 = BASE_TEMPORARY_FLAG_ID + 2;
pub const ALLOW_UPGRADE_STATS_DISPLAY_FLAG_ID: u32 = BASE_SAVED_FLAG_ID + 2;

#[derive(Debug, Deserialize, Default)]
pub struct AbsoluteWeaponConfig {
    pub log_debug_messages: Option<bool>,
    pub allow_debug_window_overlay: Option<bool>,
    pub patch_weapon_reinforcements: Option<bool>,
    pub extra_config: ErExtendRsEsdConfig,
}

pub fn get_config() -> AbsoluteWeaponConfig {
    let default_config = include_str!("../resources/absolute_weapon_config.toml");
    load_or_write_toml_config_file_alongside_dll("absolute_weapon_config.toml", "absolute_weapon.dll", default_config)
}
