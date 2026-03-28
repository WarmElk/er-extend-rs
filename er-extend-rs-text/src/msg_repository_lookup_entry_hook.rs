use crate::config::ExtraText;
use er_extend_rs_rva::{find_va_from_rva_pattern_str, HookError};
use retour::static_detour;
use std::ffi::c_void;
use windows_sys::core::PCWSTR;

static_detour! {
    static MSG_REPOSITORY_LOOKUP_ENTRY: unsafe extern "C" fn(*mut c_void, u32, u32, i32) -> PCWSTR;
}

pub fn hook_into_msg_repository_lookup_entry_with_overridden_text(er_extend_rs_text: &ExtraText) -> Result<(), HookError> {
    let msg_repository_lookup_entry_va = find_va_for_msg_repository_lookup_entry()?;
    let overridden_messages = er_extend_rs_text.generate_overridden_messages();
    unsafe {
        MSG_REPOSITORY_LOOKUP_ENTRY
            .initialize(
                std::mem::transmute::<
                    usize,
                    unsafe extern "C" fn(*mut c_void, u32, u32, i32) -> PCWSTR
                >(msg_repository_lookup_entry_va),
                move |msg_repository: *mut c_void, version: u32, category: u32, entry: i32| -> PCWSTR {
                    if let Some(overridden_text) =
                        overridden_messages
                            .get(&category)
                            .and_then(|category_map| category_map.get(&entry)) {
                        return overridden_text.as_ptr();
                    }
                    MSG_REPOSITORY_LOOKUP_ENTRY.call(msg_repository, version, category, entry)
                }
            )
            .and_then(|detour| detour.enable())
            .map_err(HookError::Retour)
    }
}

fn find_va_for_msg_repository_lookup_entry() -> Result<usize, HookError> {
    const MSG_REPOSITORY_LOOKUP_ENTRY_PATTERN: &str = "8B DA 44 8B CA 33 D2 48 8B F9 44 8D 42 6F e8 $ { ' }";
    const NUMBER_OF_CAPTURES: usize = 2;
    const CAPTURE_INDEX: usize = 1;

    find_va_from_rva_pattern_str(MSG_REPOSITORY_LOOKUP_ENTRY_PATTERN, NUMBER_OF_CAPTURES, CAPTURE_INDEX)
}
