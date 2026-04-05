use std::cmp::max;
use er_extend_rs_text::config::ExtraText;
use serde::Deserialize;

const MIN_BASE_STATE_AND_EVENT_ID: u32 = 0xFFFF;

#[derive(Debug, Deserialize, Default)]
pub struct ErExtendRsEsdConfig {
    pub extra_menu: ExtraMenu,
    pub extra_text: ExtraText,
}

#[derive(Debug, Deserialize, Default)]
pub struct ExtraMenu {
    pub base_state_and_event_id: u32,
    pub extra_menu_items: Vec<EsdSubMenuConfig>,
}

#[derive(Debug, Deserialize, Default, Clone)]
pub struct EsdSubMenuConfig {
    pub show_on_event_flag_id: Option<u32>,
    pub menu_item_text_id: u32,
    pub sub_menu_item_config: Vec<EsdSubMenuItemConfig>,
}

#[derive(Debug, Deserialize, Default, Clone)]
pub struct EsdSubMenuItemConfig {
    pub show_on_event_flag_id: Option<u32>,
    pub text_id: u32,
    pub select_flag_id: u32,
    pub confirmation_text_id: Option<u32>,
}

impl ErExtendRsEsdConfig {
    pub fn get_all_menu_item_flag_ids(&self) -> Vec<u32> {
        let mut all_flag_ids: Vec<u32> = self.extra_menu.all_menu_item_flag_ids();
        all_flag_ids.sort();
        all_flag_ids.dedup();
        all_flag_ids
    }

    pub fn is_valid(&self) -> bool {
        self.extra_text.has_text_overrides() || self.extra_menu.has_extra_menu_items()
    }
}

impl ExtraMenu {
    pub fn has_extra_menu_items(&self) -> bool {
        !self.extra_menu_items.is_empty()
    }

    pub fn get_min_base_state_and_event_id(&self) -> u32 {
        max(self.base_state_and_event_id, MIN_BASE_STATE_AND_EVENT_ID)
    }

    pub fn all_menu_item_flag_ids(&self) -> Vec<u32> {
        self.extra_menu_items.iter().flat_map(|item_config| item_config.menu_item_flag_ids()).collect()
    }

    pub fn copy_extra_menu_items(&self) -> Vec<EsdSubMenuConfig> {
        self.extra_menu_items.to_vec()
    }
}

impl EsdSubMenuConfig {
    fn menu_item_flag_ids(&self) -> Vec<u32> {
        self.sub_menu_item_config.iter().map(|item| item.select_flag_id).collect()
    }
}
