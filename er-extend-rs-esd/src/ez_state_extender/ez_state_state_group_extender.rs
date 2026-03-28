use crate::ez_state_extender::ez_state_partial_copy::{EzStateState, EzStateStateGroup};
use crate::ez_state_extender::ez_state_state_extender::EzStateStateExtender;
use crate::ez_state_extender::ez_state_transition_extender::EzStateTransitionExtender;

pub trait EzStateStateGroupExtender {
    fn find_state_with_text_id(&self, text_id: u32) -> Option<*mut EzStateState>;
    fn find_state_with_open_repository_transition(&self) -> Option<(usize, *mut EzStateState)>;
}

impl EzStateStateGroupExtender for EzStateStateGroup {
    fn find_state_with_text_id(&self, text_id: u32) -> Option<*mut EzStateState> {
        self.states
            .iter()
            .find(|&state| {
                state.contains_entry_event_with_text_id(text_id)
            })
            .map(|state| state as *const _ as *mut EzStateState)
    }

    fn find_state_with_open_repository_transition(&self) -> Option<(usize, *mut EzStateState)> {
        self.states
            .iter()
            .find_map(|state| {
                state
                    .transitions
                    .iter()
                    .map(|transition| unsafe { transition.as_ref() })
                    .position(|transition| transition.target_state_contains_open_repository_entry_event())
                    .map(|position| (position, state as *const _ as *mut EzStateState))
            })
    }
}