use crate::config::{EsdSubMenuConfig, ExtraMenu};
use crate::ez_state_enter_state_hook::{hook_into_ez_state_enter_state, HookBehavior};
use crate::ez_state_extender::ez_state_event_extender::EzStateEventFactory;
use crate::ez_state_extender::ez_state_partial_copy::{EzStateEvent, EzStateMachineImpl, EzStateState, EzStateStateGroup, EzStateTransition};
use crate::ez_state_extender::ez_state_state_extender::{EzStateStateEvents, EzStateStateExtender, EzStateStateFactory, MemoryManagement};
use crate::ez_state_extender::ez_state_state_group_extender::EzStateStateGroupExtender;
use crate::ez_state_extender::ez_state_transition_extender::EzStateTransitionFactory;
use er_extend_rs_rva::HookError;
use std::ptr::NonNull;

const SORT_CHEST_TEXT_ID_U32: u32 = 15000395;
const CANCEL_TEXT_ID_U32: u32 = 15000372;

pub fn insert_text_options_into_grace_menu(extra_menu: &ExtraMenu) -> Result<(), HookError> {
    let extra_menu_items = extra_menu.copy_extra_menu_items();
    let base_state_and_event_id = extra_menu.get_min_base_state_and_event_id();
    hook_into_ez_state_enter_state(move |state: NonNull<EzStateState>, machine: NonNull<EzStateMachineImpl>, _: usize| -> HookBehavior {
        let state_group = unsafe { machine.as_ref().state_group.as_ref() };
        let entering = state == state_group.initial_state;

        if entering {
            let mut id_offset: u32 = base_state_and_event_id;
            extra_menu_items.iter().for_each(|sub_menu_config| {
                let sub_menu_item_config_vec = &sub_menu_config.sub_menu_item_config;
                if add_menu_item_to_grace_menu(id_offset, state_group, sub_menu_config) {
                    id_offset += sub_menu_item_config_vec.len() as u32 + 2;
                }
            });
        }
        HookBehavior::CallOriginalFunctionAfterHook
    })
}

fn add_menu_item_to_grace_menu(id_offset: u32, state_group: &EzStateStateGroup, sub_menu_config: &EsdSubMenuConfig) -> bool {
    let added_item_text_id = sub_menu_config.menu_item_text_id;

    let mut item_added = false;

    let already_added = state_group.find_state_with_text_id(added_item_text_id).is_some();

    if !already_added &&
        let Some(sort_chest_state) = state_group.find_state_with_text_id(SORT_CHEST_TEXT_ID_U32) &&
        let Some((transition_index, open_repository_state)) = state_group.find_state_with_open_repository_transition() &&
        let Some((add_talk_list_data_event, add_talk_list_data_transition)) = create_state_menu_event_and_transition(id_offset, state_group, sub_menu_config) {

        item_added = true;

        if let Some(sort_chest_state) = unsafe { sort_chest_state.as_mut() } {
            sort_chest_state.append_entry_event(add_talk_list_data_event, MemoryManagement::PreserveOriginalArray);
        }

        if let Some(open_repository_transition_state) = unsafe { open_repository_state.as_mut() } {
            let static_add_talk_list_data_transition = Box::leak(Box::new(add_talk_list_data_transition));
            open_repository_transition_state.append_transition(transition_index, static_add_talk_list_data_transition, MemoryManagement::PreserveOriginalArray);
        }
    }
    item_added
}

fn create_state_menu_event_and_transition(id_offset: u32, state_group: &EzStateStateGroup, sub_menu_config: &EsdSubMenuConfig) -> Option<(EzStateEvent, EzStateTransition)> {
    let base_id = id_offset;

    let open_shop_state_id = base_id as i32;

    let sub_menu_item_config = &sub_menu_config.sub_menu_item_config;
    let shop_state_to_open: &mut EzStateState = Box::leak(Box::new(EzStateState::new(open_shop_state_id)));
    {
        shop_state_to_open.close_shop_message();
        shop_state_to_open.clear_talk_list_data();

        sub_menu_item_config.iter().enumerate().for_each(|(index, sub_menu_item)| {
            let talk_list_data_event_id = index as u32 + 1;
            match sub_menu_item.show_on_event_flag_id {
                None | Some(0) => shop_state_to_open.add_talk_list_data(talk_list_data_event_id, sub_menu_item.text_id),
                Some(flag_id) => shop_state_to_open.add_talk_list_data_if(flag_id, talk_list_data_event_id, sub_menu_item.text_id)
            }
        });

        let cancel_event_id: u32 = sub_menu_item_config.len() as u32 + 1;
        tracing::debug!("cancel_event_id: {}", cancel_event_id);
        shop_state_to_open.add_talk_list_data(cancel_event_id, CANCEL_TEXT_ID_U32);
        shop_state_to_open.show_shop_message();
    }

    let transition_handler_state_id = open_shop_state_id + 1;
    tracing::debug!("open_shop_state_id: {}, transition_handler_state_id: {}", open_shop_state_id, transition_handler_state_id);

    let shop_state_for_event_transitions = Box::leak(Box::new(EzStateState::new(transition_handler_state_id)));
    {
        sub_menu_item_config.iter().enumerate().for_each(|(index, sub_menu_item)| {
            let talk_list_data_event_id = index as u32 + 1;
            shop_state_for_event_transitions.set_event_flag_on_talk_list_data_selection(sub_menu_item.select_flag_id, talk_list_data_event_id, shop_state_to_open);
        });

        shop_state_for_event_transitions.add_back_button_control(unsafe { state_group.initial_state.as_ref() });
    }

    shop_state_to_open.add_close_shop_control(shop_state_for_event_transitions);

    let talk_event_to_open_submenu = match sub_menu_config.show_on_event_flag_id {
        None | Some(0) => EzStateEvent::new_add_talk_list_data_event(base_id, sub_menu_config.menu_item_text_id),
        Some(flag_id) => EzStateEvent::new_add_talk_list_data_if_event(flag_id, base_id, sub_menu_config.menu_item_text_id)
    };
    let transition_to_open_shop_state = EzStateTransition::new_talk_list_data(shop_state_to_open, base_id);

    Some((talk_event_to_open_submenu, transition_to_open_shop_state))
}
