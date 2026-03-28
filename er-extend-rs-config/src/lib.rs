use serde::de::DeserializeOwned;
use std::path::{Path, PathBuf};
use windows_sys::Win32::System::LibraryLoader::{GetModuleFileNameW, GetModuleHandleW};

const MAX_WINDOWS_PATH_LENGTH: usize = 32767;

pub fn load_toml_config_file_from_alongside_dll<C>(config_file_name: &str, alongside_dll_name: &str) -> C
where
    C: Default + DeserializeOwned {

    find_config_file_name(config_file_name, alongside_dll_name)
        .and_then(|config_path| std::fs::read_to_string(config_path).ok())
        .and_then(|config| toml::from_str(config.as_str()).ok())
        .unwrap_or_default()
}

pub fn load_or_write_toml_config_file_alongside_dll<C>(config_file_name: &str, alongside_dll_name: &str, default_content: &str) -> C
where
    C: Default + DeserializeOwned {

    let Some(absolute_config_file) = find_config_file_name(config_file_name, alongside_dll_name) else {
        return toml::from_str(default_content).unwrap_or_default();
    };

    let content = if absolute_config_file.exists() {
        std::fs::read_to_string(absolute_config_file).ok()
    }
    else {
        std::fs::write(&absolute_config_file, default_content).ok();
        Some(default_content.to_string())
    };

    content
        .and_then(|config| toml::from_str(config.as_str()).ok())
        .unwrap_or_default()
}

fn find_config_file_name(config_file_name: &str, alongside_dll_name: &str) -> Option<PathBuf> {
    let alongside_dll_name_utf_16 = alongside_dll_name.encode_utf16().chain(Some(0)).collect::<Vec<_>>();

    let h_module = unsafe { GetModuleHandleW(alongside_dll_name_utf_16.as_ptr()) };

    let mut module_path = [0u16; MAX_WINDOWS_PATH_LENGTH];
    let module_path_length = unsafe { GetModuleFileNameW(h_module, module_path.as_mut_ptr(), module_path.len() as u32) };

    String::from_utf16(&module_path[..module_path_length as usize])
        .ok()
        .and_then(|module_path_name|
            Path::new(&module_path_name)
                .parent()
                .map(|parent| parent.join(config_file_name))
        )
}