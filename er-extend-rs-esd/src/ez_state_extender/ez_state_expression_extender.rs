use crate::ez_state_extender::ez_state_partial_copy::EzStateExpression;
use crate::ez_state_extender::stl_partial_copy::DynamicSizeSpan;

const EQUALS: u8 = 0x95;
const U32_INDICATOR: u8 = 0x82;
const EXPRESSION_END_INDICATOR: u8 = 0xA1;

const U32_INDICATOR_START: &[u8] = &[U32_INDICATOR];
const EXPRESSION_INDICATOR_END: &[u8] = &[EXPRESSION_END_INDICATOR];

const TALK_LIST_EVENT_ID_EVALUATOR_START: &[u8] = &[0x57, 0x84, U32_INDICATOR];
const TALK_LIST_EVENT_ID_EVALUATOR_END: &[u8] = &[EQUALS, EXPRESSION_END_INDICATOR];

const CLOSE_SHOP_MENU_EVALUATOR: &[u8] = &[
    0x7b,  // 59 (CheckSpecificPersonMenuIsOpen)
    0x41,  // 1
    0x40,  // 0
    0x86,  // call with 2 args
    0x41,  // 1
    EQUALS,  // ==
    0x7a,  // 58 (CheckSpecificPersonGenericDialogIsOpen)
    0x40,  // 0
    0x85,  // call with 1 arg
    0x40,  // 0
    EQUALS,  // ==
    0x98,  // &&
    0x40,  // 0
    EQUALS,  // ==
    EXPRESSION_END_INDICATOR   // end
];

const HANDLE_BACK_BUTTON_EVALUATOR: &[u8] = &[0x41, EXPRESSION_END_INDICATOR];
const EZ_STATE_PUSH_1: &[u8] = &[U32_INDICATOR, 0x01, 0x00, 0x00, 0x00, EXPRESSION_END_INDICATOR];

const SHOW_SHOP_MESSAGE_ARGS: &[EzStateExpression] = &[EzStateExpression::from_static_slice(EZ_STATE_PUSH_1)];

const GET_EVENT_FLAG_ARG_0_START: &[u8] = &[0x4F, U32_INDICATOR];
const GET_EVENT_FLAG_ARG_0_END: &[u8] = &[0x85, EXPRESSION_END_INDICATOR];
const DIALOG_CONFIRMED_EVALUATOR: &[u8] = &[0xB9, U32_INDICATOR, 0x00, 0x00, 0x00, 0x00, EQUALS, EXPRESSION_END_INDICATOR];
const CALL_DONE_EVALUATOR: &[u8] = &[0xB9, 0xBA, 0x96, EXPRESSION_END_INDICATOR];

pub trait EzStateExpressionFactory {
    fn new_talk_data_event_id_evaluator(talk_list_event_id: u32) -> Self;
    fn new_get_event_flag_expression(flag_id: u32) -> Self;
    fn new_close_shop_menu_evaluator() -> Self;
    fn new_handle_back_button_evaluator() -> Self;
    fn new_dialog_confirmed_evaluator() -> Self;
    fn new_call_done_evaluator() -> Self;
}

pub trait EzStateExpressionExtender {
    fn new_shop_message_args() -> DynamicSizeSpan<EzStateExpression>;
    fn generate_plain_u32_indicator(plain_u32: u32) -> &'static mut Vec<u8>;
    fn generate_u32_minus_1_equivalent_indicator() -> &'static mut Vec<u8>;
    fn to_u32_argument(&self) -> Option<u32>;
}

impl EzStateExpressionFactory for EzStateExpression {
    fn new_talk_data_event_id_evaluator(talk_list_event_id: u32) -> Self {
        let expression_bytes = Box::leak(Box::new([
            TALK_LIST_EVENT_ID_EVALUATOR_START.to_vec(),
            talk_list_event_id.to_le_bytes().to_vec(),
            TALK_LIST_EVENT_ID_EVALUATOR_END.to_vec()
        ].concat()));
        EzStateExpression::from_static_slice(expression_bytes.as_slice())
    }

    fn new_get_event_flag_expression(flag_id: u32) -> Self {
        let expression_bytes = Box::leak(Box::new([
            GET_EVENT_FLAG_ARG_0_START.to_vec(),
            flag_id.to_le_bytes().to_vec(),
            GET_EVENT_FLAG_ARG_0_END.to_vec()
        ].concat()));
        EzStateExpression::from_static_slice(expression_bytes.as_slice())
    }

    fn new_close_shop_menu_evaluator() -> Self {
        EzStateExpression::from_static_slice(CLOSE_SHOP_MENU_EVALUATOR)
    }

    fn new_handle_back_button_evaluator() -> Self {
        EzStateExpression::from_static_slice(HANDLE_BACK_BUTTON_EVALUATOR)
    }

    fn new_dialog_confirmed_evaluator() -> Self {
        EzStateExpression::from_static_slice(DIALOG_CONFIRMED_EVALUATOR)
    }

    fn new_call_done_evaluator() -> Self {
        EzStateExpression::from_static_slice(CALL_DONE_EVALUATOR)
    }
}

impl EzStateExpressionExtender for EzStateExpression {
    fn new_shop_message_args() -> DynamicSizeSpan<EzStateExpression> {
        DynamicSizeSpan::from_static_slice(SHOW_SHOP_MESSAGE_ARGS)
    }

    fn generate_plain_u32_indicator(plain_u32: u32) -> &'static mut Vec<u8> {
        Box::leak(Box::new([
            U32_INDICATOR_START.to_vec(),
            plain_u32.to_le_bytes().to_vec(),
            EXPRESSION_INDICATOR_END.to_vec(),
        ].concat()))
    }

    fn generate_u32_minus_1_equivalent_indicator() -> &'static mut Vec<u8> {
        Self::generate_plain_u32_indicator(u32::MAX)
    }

    fn to_u32_argument(&self) -> Option<u32> {
        let bytes = self.as_slice();
        if bytes.len() == 6 && bytes[0] == U32_INDICATOR && bytes[5] == EXPRESSION_END_INDICATOR {
            Some(u32::from_le_bytes([bytes[1], bytes[2], bytes[3], bytes[4]]))
        }
        else {
            None
        }
    }
}

