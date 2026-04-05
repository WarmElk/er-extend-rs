use crate::ez_state_extender::ez_state_expression_extender::EzStateExpressionFactory;
use crate::ez_state_extender::ez_state_partial_copy::{EzStateExpression, EzStateState, EzStateTransition};
use crate::ez_state_extender::ez_state_state_extender::EzStateStateExtender;
use crate::ez_state_extender::stl_partial_copy::DynamicSizeSpan;
use std::ptr::NonNull;

pub trait EzStateTransitionFactory {
    fn new(target_state: &EzStateState, evaluator: EzStateExpression) -> Self;
    fn new_talk_list_data(target_state: &EzStateState, event_id: u32) -> Self;
    fn new_dialog_confirmed_transition(target_state: &EzStateState) -> Self;
    fn new_call_done_transition(target_state: &EzStateState) -> Self;
}

pub trait EzStateTransitionExtender {
    fn target_state_contains_open_repository_entry_event(&self) -> bool;
}

impl EzStateTransitionFactory for EzStateTransition {
    fn new(target_state: &EzStateState, evaluator: EzStateExpression) -> Self {
        Self {
            target_state: Some(NonNull::from_ref(target_state)),
            pass_events: DynamicSizeSpan::empty(),
            sub_transitions: DynamicSizeSpan::empty(),
            evaluator,
        }
    }

    fn new_talk_list_data(target_state: &EzStateState, event_id: u32) -> Self {
        Self {
            target_state: Some(NonNull::from_ref(target_state)),
            pass_events: DynamicSizeSpan::empty(),
            sub_transitions: DynamicSizeSpan::empty(),
            evaluator: EzStateExpression::new_talk_data_event_id_evaluator(event_id),
        }
    }

    fn new_dialog_confirmed_transition(target_state: &EzStateState) -> Self {
        Self {
            target_state: Some(NonNull::from_ref(target_state)),
            pass_events: DynamicSizeSpan::empty(),
            sub_transitions: DynamicSizeSpan::empty(),
            evaluator: EzStateExpression::new_dialog_confirmed_evaluator(),
        }
    }

    fn new_call_done_transition(target_state: &EzStateState) -> Self {
        Self {
            target_state: Some(NonNull::from_ref(target_state)),
            pass_events: DynamicSizeSpan::empty(),
            sub_transitions: DynamicSizeSpan::empty(),
            evaluator: EzStateExpression::new_call_done_evaluator(),
        }
    }
}

impl EzStateTransitionExtender for EzStateTransition {
    fn target_state_contains_open_repository_entry_event(&self) -> bool {
        self
            .target_state
            .as_ref()
            .map(|state| unsafe{ state.as_ref() })
            .is_some_and(|state| state.contains_open_repository_entry_event())
    }
}