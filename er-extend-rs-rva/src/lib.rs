use fromsoftware_shared::Program;
use pelite::pattern;
use pelite::pattern::{Atom, ParsePatError};
use pelite::pe64::Pe;

#[derive(Debug)]
pub enum HookError {
    Retour(retour::Error),
    Pelite(pelite::Error),
    ParsePatError(ParsePatError),
    NoDataToHook,
}

pub fn find_va_from_rva_pattern_str(pattern_str: &str, number_of_captures: usize, capture_index: usize) -> Result<usize, HookError> {
    let pattern = pattern::parse(pattern_str).map_err(HookError::ParsePatError)?;

    find_va_from_rva_pattern(&pattern, number_of_captures, capture_index)
}

pub fn find_va_from_rva_pattern(pattern: &[Atom], number_of_captures: usize, capture_index: usize) -> Result<usize, HookError> {
    let rva = find_rva(pattern, number_of_captures, capture_index);
    tracing::debug!("pattern: {:?}, rva: {:#x}", pattern, rva);

    let va= Program::current().rva_to_va(rva).map_err(|error| {
        tracing::error!("rva to va error: {:?}", error);
        HookError::Pelite(error)
    })?;

    tracing::debug!("va: {:#x}", va);
    Ok(va as usize)
}

fn find_rva(pattern: &[Atom], number_of_captures: usize, capture_index: usize) -> u32 {
    let mut matches = Program::current().scanner().matches_code(pattern);

    let mut captures = vec![0u32; number_of_captures];
    if matches.next(&mut captures) {
        captures[capture_index]
    } else {
        0
    }
}
