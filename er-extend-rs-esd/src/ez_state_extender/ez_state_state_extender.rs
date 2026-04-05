use crate::ez_state_extender::ez_state_event_extender::EZ_STATE_COMMAND_OPEN_REPOSITORY;
use crate::ez_state_extender::ez_state_event_extender::{EzStateEventFactory, EZ_STATE_COMMAND_ADD_TALK_LIST_DATA, EZ_STATE_COMMAND_OPEN_SHOP};
use crate::ez_state_extender::ez_state_expression_extender::{EzStateExpressionExtender, EzStateExpressionFactory};
use crate::ez_state_extender::ez_state_partial_copy::{EzStateEvent, EzStateExpression, EzStateState, EzStateTransition};
use crate::ez_state_extender::ez_state_transition_extender::EzStateTransitionFactory;
use crate::ez_state_extender::stl_partial_copy::DynamicSizeSpan;
use std::ptr::NonNull;

pub enum MemoryManagement {
    DeallocateOriginalArray,
    PreserveOriginalArray,
}

pub trait EzStateStateFactory {
    fn new(id: i32) -> Self;
}

pub trait EzStateStateExtender {
    fn contains_entry_event_with_text_id(&self, text_id: u32) -> bool;
    fn contains_open_repository_entry_event(&self) -> bool;
    fn append_entry_event(&mut self, entry_event: EzStateEvent, memory_management: MemoryManagement);
    fn append_transition(&mut self, transition_index: usize, transition: &EzStateTransition, memory_management: MemoryManagement);
}

pub trait EzStateStateEvents {
    fn close_shop_message(&mut self);
    fn clear_talk_list_data(&mut self);
    fn show_shop_message(&mut self);

    fn add_talk_list_data(&mut self, event_id: u32, text_id: u32);
    fn add_talk_list_data_if(&mut self, flag_id: u32, event_id: u32, text_id: u32);
    fn add_back_button_control(&mut self, target_state: &EzStateState);
    fn add_close_shop_control(&mut self, transition_state: &EzStateState);

    fn set_event_flag_on_talk_list_data_selection(&mut self, flag_id_to_set: u32, talk_list_data_event_id: u32, target_state: &EzStateState, confirm_text_id: Option<u32>);
}

impl EzStateStateFactory for EzStateState {
    fn new(id: i32) -> Self {
        Self {
            id,
            transitions: DynamicSizeSpan::empty(),
            entry_events: DynamicSizeSpan::empty(),
            exit_events: DynamicSizeSpan::empty(),
            while_events: DynamicSizeSpan::empty(),
        }
    }
}

impl EzStateStateExtender for EzStateState {
    fn contains_entry_event_with_text_id(&self, text_id: u32) -> bool {
        self
            .entry_events
            .iter()
            .filter(|event| event.command == EZ_STATE_COMMAND_ADD_TALK_LIST_DATA)
            .map(|event| event.arguments.as_slice())
            .filter_map(|args| args.get(1))
            .filter_map(EzStateExpression::to_u32_argument)
            .any(|id| id == text_id)
    }

    fn contains_open_repository_entry_event(&self) -> bool {
        self
            .entry_events
            .iter()
            .any(|event| event.command == EZ_STATE_COMMAND_OPEN_REPOSITORY)
    }

    fn append_entry_event(&mut self, entry_event: EzStateEvent, memory_management: MemoryManagement) {
        let old_count = self.entry_events.len();
        let new_count = old_count + 1;

        let layout =
            std::alloc::Layout::array::<EzStateEvent>(new_count).unwrap();

        unsafe {
            let alloc = std::alloc::alloc(layout) as *mut EzStateEvent;

            std::ptr::copy_nonoverlapping(
                self.entry_events.as_ptr(),
                alloc,
                old_count,
            );

            std::ptr::write(
                alloc.add(old_count),
                entry_event,
            );

            if let MemoryManagement::DeallocateOriginalArray = memory_management && old_count > 0 && !self.entry_events.as_ptr().is_null() {
                let old_layout = std::alloc::Layout::array::<EzStateEvent>(old_count).unwrap();
                std::alloc::dealloc(self.entry_events.as_ptr() as *mut u8, old_layout);
            }

            self.entry_events = DynamicSizeSpan::from_raw_parts(alloc, new_count);
        }
    }

    fn append_transition(&mut self, transition_index: usize, transition: &EzStateTransition, memory_management: MemoryManagement) {
        let old_count = self.transitions.len();
        let new_count = old_count + 1;

        let layout = std::alloc::Layout::array::<NonNull<EzStateTransition>>(new_count).unwrap();

        unsafe {
            let alloc = std::alloc::alloc(layout) as *mut NonNull<EzStateTransition>;

            if transition_index > 0 {
                std::ptr::copy_nonoverlapping(self.transitions.as_ptr(), alloc, transition_index);
            }
            std::ptr::write(alloc.add(transition_index), NonNull::from_ref(transition));
            if transition_index < old_count {
                std::ptr::copy_nonoverlapping(
                    self.transitions.as_ptr().add(transition_index),
                    alloc.add(transition_index + 1),
                    old_count - transition_index
                );
            }

            if let MemoryManagement::DeallocateOriginalArray = memory_management && old_count > 0 && !self.transitions.as_ptr().is_null() {
                let old_layout = std::alloc::Layout::array::<NonNull<EzStateTransition>>(old_count).unwrap();
                std::alloc::dealloc(self.transitions.as_ptr() as *mut u8, old_layout);
            }

            self.transitions = DynamicSizeSpan::from_raw_parts(alloc, new_count);
        }
    }
}

impl EzStateStateEvents for EzStateState {
    fn close_shop_message(&mut self) {
        let close_shop = EzStateEvent::new_close_shop_event();
        self.append_entry_event(close_shop, MemoryManagement::DeallocateOriginalArray);
    }

    fn clear_talk_list_data(&mut self) {
        let clear_talk_list = EzStateEvent::new_clear_talk_list_data_event();
        self.append_entry_event(clear_talk_list, MemoryManagement::DeallocateOriginalArray);
    }

    fn show_shop_message(&mut self) {
        let open_shop = EzStateEvent::new(EZ_STATE_COMMAND_OPEN_SHOP, EzStateExpression::new_shop_message_args());
        self.append_entry_event(open_shop, MemoryManagement::DeallocateOriginalArray);
    }

    fn add_talk_list_data(&mut self, event_id: u32, text_id: u32) {
        let event = EzStateEvent::new_add_talk_list_data_event(event_id, text_id);
        self.append_entry_event(event, MemoryManagement::DeallocateOriginalArray);
    }

    fn add_talk_list_data_if(&mut self, flag_id: u32, event_id: u32, text_id: u32) {
        let event = EzStateEvent::new_add_talk_list_data_if_event(flag_id, event_id, text_id);
        self.append_entry_event(event, MemoryManagement::DeallocateOriginalArray);
    }

    fn add_back_button_control(&mut self, target_state: &EzStateState) {
        let handle_back_button_transition = Box::leak(Box::new(EzStateTransitionFactory::new(target_state, EzStateExpression::new_handle_back_button_evaluator())));
        self.append_transition(self.transitions.len(), handle_back_button_transition, MemoryManagement::DeallocateOriginalArray);
    }

    fn add_close_shop_control(&mut self, transition_state: &EzStateState) {
        let close_shop_menu_transition = Box::leak(Box::new(EzStateTransition::new(transition_state, EzStateExpression::new_close_shop_menu_evaluator())));
        self.append_transition(0, close_shop_menu_transition, MemoryManagement::DeallocateOriginalArray);
    }

    fn set_event_flag_on_talk_list_data_selection(&mut self, flag_id_to_set: u32, talk_list_data_event_id: u32, target_state: &EzStateState, confirm_text_id: Option<u32>) {
        let set_event_flag_state_id = self.id + talk_list_data_event_id as i32;

        tracing::debug!("sub_menu_event_id: {}, set_event_flag_state_id: {}", talk_list_data_event_id, set_event_flag_state_id);

        let set_event_flag_state: &mut EzStateState = Box::leak(Box::new(EzStateState::new(set_event_flag_state_id)));

        let set_event_flag_event = EzStateEvent::new_set_event_flag_event(flag_id_to_set);

        let handle_set_flag_state_transition = Box::leak(Box::new(EzStateTransition::new_talk_list_data(target_state, talk_list_data_event_id)));

        set_event_flag_state.append_entry_event(set_event_flag_event, MemoryManagement::DeallocateOriginalArray);
        set_event_flag_state.append_transition(set_event_flag_state.transitions.len(), handle_set_flag_state_transition, MemoryManagement::DeallocateOriginalArray);

        let selection_state = match confirm_text_id {
            None | Some(0) => set_event_flag_state,
            Some(text_id) => create_confirmation_state(text_id, set_event_flag_state, target_state)
        };

        let sub_menu_selected_transition = Box::leak(Box::new(EzStateTransition::new_talk_list_data(selection_state, talk_list_data_event_id)));
        self.append_transition(0, sub_menu_selected_transition, MemoryManagement::DeallocateOriginalArray);
    }
}

fn create_confirmation_state<'a>(confirmation_text_id: u32, set_event_flag_state: &EzStateState, original_target_state: &EzStateState) -> &'a EzStateState {
    let handle_confirmation_state_id = set_event_flag_state.id + 1;
    let handle_confirmation_state: &mut EzStateState = Box::leak(Box::new(EzStateState::new(handle_confirmation_state_id)));

    let handle_confirmation_state_event = EzStateEvent::new_grace_confirmation_dialog_event(confirmation_text_id);

    handle_confirmation_state.append_entry_event(handle_confirmation_state_event, MemoryManagement::DeallocateOriginalArray);

    let dialog_confirmed_transition = Box::leak(Box::new(EzStateTransition::new_dialog_confirmed_transition(set_event_flag_state)));
    let call_done_transition = Box::leak(Box::new(EzStateTransition::new_call_done_transition(original_target_state)));

    handle_confirmation_state.append_transition(0, dialog_confirmed_transition, MemoryManagement::DeallocateOriginalArray);
    handle_confirmation_state.append_transition(1, call_done_transition, MemoryManagement::DeallocateOriginalArray);

    handle_confirmation_state
}
