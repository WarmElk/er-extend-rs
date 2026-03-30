use er_extend_rs_config::load_or_write_toml_config_file_alongside_dll;
use er_extend_rs_esd::config::ErExtendRsEsdConfig;
use serde::Deserialize;

const BASE_MOD_ID: u32 = 1061470000;
const BASE_TEMPORARY_FLAG_ID: u32 = BASE_MOD_ID + 5000;

pub const UPGRADE_ALL_WEAPONS_FLAG_ID: u32 = BASE_TEMPORARY_FLAG_ID + 1;
pub const TOGGLE_UPGRADE_STATS_DISPLAY_FLAG_ID: u32 = BASE_TEMPORARY_FLAG_ID + 2;

#[derive(Debug, Deserialize, Default)]
pub struct AbsoluteWeaponConfig {
    pub log_debug_messages: Option<bool>,
    pub allow_debug_window_overlay: Option<bool>,
    pub patch_weapon_reinforcements: Option<bool>,
    pub load_menu_on_flag_id: Option<u32>,
    pub extra_config: ErExtendRsEsdConfig,
}

impl AbsoluteWeaponConfig {
    pub fn filter_out_flag_id(&mut self, flag_id: u32) {
        self.extra_config.extra_menu.extra_menu_items.iter_mut().for_each(|item| {
            item.sub_menu_item_config.retain(|item| item.flag_id != flag_id);
        });
    }
}

pub fn get_config() -> AbsoluteWeaponConfig {
    let default_config = include_str!("../resources/absolute_weapon_config.toml");
    load_or_write_toml_config_file_alongside_dll("absolute_weapon_config.toml", "absolute_weapon.dll", default_config)
}
