pub mod msg_repository_lookup_entry_hook;
pub mod config;

use crate::config::ExtraText;
use crate::msg_repository_lookup_entry_hook::hook_into_msg_repository_lookup_entry_with_overridden_text;
use er_extend_rs_config::load_toml_config_file_from_alongside_dll;
use er_extend_rs_rva::HookError;

const DEFAULT_ER_EXTEND_RS_TEXT_CONFIG_FILE: &str = "er-extend-rs-text.toml";

pub fn initialize_er_extend_rs_text_default_config_alongside_dll(alongside_dll_name: &str) -> Result<(), HookError> {
    initialize_er_extend_rs_text_config_alongside_dll(DEFAULT_ER_EXTEND_RS_TEXT_CONFIG_FILE, alongside_dll_name)
}

pub fn initialize_er_extend_rs_text_config_alongside_dll(er_extend_rs_text_filename: &str, alongside_dll_name: &str) -> Result<(), HookError> {
    let extra_text: ExtraText = load_toml_config_file_from_alongside_dll(er_extend_rs_text_filename, alongside_dll_name);
    initialize_er_extend_rs_text(&extra_text)
}

pub fn initialize_er_extend_rs_text(extra_text: &ExtraText) -> Result<(), HookError> {
    if !extra_text.has_text_overrides() {
        return Err(HookError::NoDataToHook);
    }
    hook_into_msg_repository_lookup_entry_with_overridden_text(extra_text)
}

// TODO: Add dllMain or an example to have the option to deploy this as a dll, picking up the default config
