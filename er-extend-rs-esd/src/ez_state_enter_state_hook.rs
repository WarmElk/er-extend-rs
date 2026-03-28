use crate::ez_state_extender::ez_state_partial_copy::{EzStateMachineImpl, EzStateState};
use er_extend_rs_rva::{find_va_from_rva_pattern_str, HookError};
use retour::static_detour;
use std::ptr::NonNull;

static_detour! {
    static EZ_STATE_ENTER_STATE: extern "C" fn(NonNull<EzStateState>, NonNull<EzStateMachineImpl>, usize);
}

pub enum HookBehavior {
    CallOriginalFunctionAfterHook,
    SkipOriginalFunctionCall,
}

pub fn hook_into_ez_state_enter_state<F>(ez_state_enter_state_hook: F) -> Result<(), HookError>
where
    F: Fn(NonNull<EzStateState>, NonNull<EzStateMachineImpl>, usize) -> HookBehavior + Send + Sync + 'static,
{
    let ez_state_enter_state_va = find_va_for_ez_state_enter_state()?;
    unsafe {
        EZ_STATE_ENTER_STATE
            .initialize(
                std::mem::transmute::<
                    usize,
                    extern "C" fn(NonNull<EzStateState>, NonNull<EzStateMachineImpl>, usize)
                >(ez_state_enter_state_va),
                move |state: NonNull<EzStateState>, machine: NonNull<EzStateMachineImpl>, unk: usize| {
                    let hook_behavior = ez_state_enter_state_hook(state, machine, unk);
                    if let HookBehavior::CallOriginalFunctionAfterHook = hook_behavior {
                        EZ_STATE_ENTER_STATE.call(state, machine, unk);
                    }
                },
            )
            .and_then(|detour| detour.enable())
            .map_err(HookError::Retour)
    }
}


fn find_va_for_ez_state_enter_state() -> Result<usize, HookError> {
    const EZ_STATE_ENTER_STATE_PATTERN: &str = "80 7e 18 00 74 15 4c 8d 44 24 40 48 8b d6 48 8b 4e 20 e8 $ { ' }";
    const NUMBER_OF_CAPTURES: usize = 2;
    const CAPTURE_INDEX: usize = 1;

    find_va_from_rva_pattern_str(EZ_STATE_ENTER_STATE_PATTERN, NUMBER_OF_CAPTURES, CAPTURE_INDEX)
}