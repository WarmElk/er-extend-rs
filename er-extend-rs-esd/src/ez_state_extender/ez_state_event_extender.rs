use crate::ez_state_extender::ez_state_expression_extender::{EzStateExpressionExtender, EzStateExpressionFactory};
use crate::ez_state_extender::ez_state_partial_copy::{EzStateEvent, EzStateEventCommand, EzStateExpression};
use crate::ez_state_extender::stl_partial_copy::DynamicSizeSpan;

pub const EZ_STATE_COMMAND_OPEN_SHOP: EzStateEventCommand = EzStateEventCommand {
    bank: 1,
    id: 10,
};

pub const EZ_STATE_COMMAND_SET_EVENT_FLAG: EzStateEventCommand = EzStateEventCommand {
    bank: 1,
    id: 11,
};

pub const EZ_STATE_COMMAND_CLOSE_SHOP: EzStateEventCommand = EzStateEventCommand {
    bank: 1,
    id: 12,
};

pub const EZ_STATE_COMMAND_ADD_TALK_LIST_DATA: EzStateEventCommand = EzStateEventCommand {
    bank: 1,
    id: 19,
};

pub const EZ_STATE_COMMAND_CLEAR_TALK_LIST_DATA: EzStateEventCommand = EzStateEventCommand {
    bank: 1,
    id: 20,
};

pub const EZ_STATE_COMMAND_OPEN_REPOSITORY: EzStateEventCommand = EzStateEventCommand {
    bank: 1,
    id: 30,
};

pub const EZ_STATE_COMMAND_ADD_TALK_LIST_DATA_IF: EzStateEventCommand = EzStateEventCommand {
    bank: 5,
    id: 19,
};


pub trait EzStateEventFactory {
    fn new(command: EzStateEventCommand, arguments: DynamicSizeSpan<EzStateExpression>) -> Self;
    fn new_no_args_command_event(command: EzStateEventCommand) -> Self;
    fn new_add_talk_list_data_event(talk_list_event_id: u32, option_text_id: u32) -> Self;
    fn new_add_talk_list_data_if_event(flag_id: u32, talk_list_event_id: u32, option_text_id: u32) -> Self;
    fn new_set_event_flag_event(flag_id: u32) -> Self;
    fn new_close_shop_event() -> Self;
    fn new_clear_talk_list_data_event() -> Self;
}

pub trait EzStateEventExtender {
    fn generate_add_talk_list_data_arguments(talk_list_event_id: u32, option_text_id: u32) -> &'static mut [EzStateExpression; 3];
    fn generate_add_talk_list_data_if_arguments(flag_id: u32, talk_list_event_id: u32, option_text_id: u32) -> &'static mut [EzStateExpression; 4];
    fn generate_set_event_flag_arguments(flag_id: u32) -> &'static mut [EzStateExpression; 2];
}

impl EzStateEventFactory for EzStateEvent {
    fn new(command: EzStateEventCommand, arguments: DynamicSizeSpan<EzStateExpression>) -> Self {
        Self {
            command,
            arguments,
        }
    }

    fn new_no_args_command_event(command: EzStateEventCommand) -> Self {
        Self {
            command,
            arguments: DynamicSizeSpan::empty(),
        }
    }

    fn new_add_talk_list_data_event(talk_list_event_id: u32, option_text_id: u32) -> Self {
        let add_talk_list_data_arguments = Self::generate_add_talk_list_data_arguments(talk_list_event_id, option_text_id);

        Self {
            command: EZ_STATE_COMMAND_ADD_TALK_LIST_DATA,
            arguments: DynamicSizeSpan::from_static_slice(
                add_talk_list_data_arguments,
            ),
        }
    }

    fn new_add_talk_list_data_if_event(flag_id: u32, talk_list_event_id: u32, option_text_id: u32) -> Self {
        let add_talk_list_data_if_arguments = Self::generate_add_talk_list_data_if_arguments(flag_id, talk_list_event_id, option_text_id);

        Self {
            command: EZ_STATE_COMMAND_ADD_TALK_LIST_DATA_IF,
            arguments: DynamicSizeSpan::from_static_slice(
                add_talk_list_data_if_arguments,
            ),
        }
    }

    fn new_set_event_flag_event(flag_id: u32) -> Self {
        let upgrade_add_talk_list_data_arguments = Self::generate_set_event_flag_arguments(flag_id);

        Self {
            command: EZ_STATE_COMMAND_SET_EVENT_FLAG,
            arguments: DynamicSizeSpan::from_static_slice(
                upgrade_add_talk_list_data_arguments,
            ),
        }
    }

    fn new_close_shop_event() -> Self {
        Self::new_no_args_command_event(EZ_STATE_COMMAND_CLOSE_SHOP)
    }

    fn new_clear_talk_list_data_event() -> Self {
        Self::new_no_args_command_event(EZ_STATE_COMMAND_CLEAR_TALK_LIST_DATA)
    }
}

impl EzStateEventExtender for EzStateEvent {
    fn generate_add_talk_list_data_arguments(talk_list_event_id: u32, option_text_id: u32) -> &'static mut [EzStateExpression; 3] {
        let talk_list_event_id_indicator = EzStateExpression::generate_plain_u32_indicator(talk_list_event_id);
        let talk_list_text_id_indicator = EzStateExpression::generate_plain_u32_indicator(option_text_id);
        let talk_list_end_indicator = EzStateExpression::generate_u32_minus_1_equivalent_indicator();

        Box::leak(Box::new([
            EzStateExpression::from_static_slice(talk_list_event_id_indicator.as_slice()),
            EzStateExpression::from_static_slice(talk_list_text_id_indicator.as_slice()),
            EzStateExpression::from_static_slice(talk_list_end_indicator.as_slice()),
        ]))
    }

    fn generate_add_talk_list_data_if_arguments(flag_id: u32, talk_list_event_id: u32, option_text_id: u32) -> &'static mut [EzStateExpression; 4] {
        let get_event_flag_expression = EzStateExpression::new_get_event_flag_expression(flag_id);
        let talk_list_event_id_indicator = EzStateExpression::generate_plain_u32_indicator(talk_list_event_id);
        let talk_list_text_id_indicator = EzStateExpression::generate_plain_u32_indicator(option_text_id);
        let talk_list_end_indicator = EzStateExpression::generate_u32_minus_1_equivalent_indicator();

        Box::leak(Box::new([
            get_event_flag_expression,
            EzStateExpression::from_static_slice(talk_list_event_id_indicator.as_slice()),
            EzStateExpression::from_static_slice(talk_list_text_id_indicator.as_slice()),
            EzStateExpression::from_static_slice(talk_list_end_indicator.as_slice()),
        ]))
    }

    fn generate_set_event_flag_arguments(flag_id: u32) -> &'static mut [EzStateExpression; 2] {
        let event_flag_indicator = EzStateExpression::generate_plain_u32_indicator(flag_id);
        let on_indicator = EzStateExpression::generate_plain_u32_indicator(1);

        Box::leak(Box::new([
            EzStateExpression::from_static_slice(event_flag_indicator.as_slice()),
            EzStateExpression::from_static_slice(on_indicator.as_slice()),
        ]))
    }
}

