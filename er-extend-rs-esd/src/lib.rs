pub mod ez_state_enter_state_hook;
pub mod ez_state_extender;
pub mod esd_grace_menu_enhancement;
pub mod config;

use crate::config::ErExtendRsEsdConfig;
use crate::esd_grace_menu_enhancement::insert_text_options_into_grace_menu;
use er_extend_rs_config::load_toml_config_file_from_alongside_dll;
use er_extend_rs_rva::HookError;
use er_extend_rs_text::initialize_er_extend_rs_text;

const ER_EXTEND_RS_ESD_CONFIG_FILE: &str = "er-extend-rs-esd-config.toml";

pub fn initialize_er_extend_rs_esd_from_config_alongside_dll(alongside_dll_name: &str) -> Result<(), HookError> {
    let er_extend_rs_esd_config: ErExtendRsEsdConfig = load_toml_config_file_from_alongside_dll(ER_EXTEND_RS_ESD_CONFIG_FILE, alongside_dll_name);
    initialize_er_extend_rs_esd_from_config(&er_extend_rs_esd_config)
}

pub fn initialize_er_extend_rs_esd_from_config(er_extend_rs_esd_config: &ErExtendRsEsdConfig) -> Result<(), HookError> {
    let extra_text = &er_extend_rs_esd_config.extra_text;
    if extra_text.has_text_overrides() {
        initialize_er_extend_rs_text(extra_text)?;
    }
    insert_text_options_into_grace_menu(&er_extend_rs_esd_config.extra_menu)
}

// TODO: Example for and changing the upgrades when running in Reborn
// TODO: Add README.md
// TODO: Add dllMain or an example to have the option to deploy this as a dll, picking up the default config