use eldenring::cs::{EzStateInvokeError, MenuType, TalkScript};
use eldenring::ez_state::EzStateValue;

pub trait TalkScriptExtender {
    fn clear_talk_list_data(&mut self) -> Result<(), EzStateInvokeError>;
    fn add_talk_list_data(&mut self, index: i32, text_id: i32, unk1: i32) -> Result<(), EzStateInvokeError>;
    fn show_shop_message(&mut self, unk1: i32) -> Result<(), EzStateInvokeError>;
    fn open_regular_shop(&mut self, start_id_inclusive: i32, end_id_inclusive: i32) -> Result<(), EzStateInvokeError>;
    fn open_sell_shop(&mut self, start_id_inclusive: i32, end_id_inclusive: i32) -> Result<(), EzStateInvokeError>;
    fn get_talk_list_menu_result(&mut self) -> Result<EzStateValue, EzStateInvokeError>;
    fn check_specific_person_menu_is_open(&mut self, menu_type: MenuType, id: i32) -> Result<EzStateValue, EzStateInvokeError>;
    fn check_specific_person_generic_dialog_is_open(&mut self, id: i32) -> Result<EzStateValue, EzStateInvokeError>;
}

enum TalkScriptId {
    ShowShopMessage = 10,
    AddTalkListData = 19,
    ClearTalkListData = 20,

    OpenRegularShop = 22,
    GetTalkListMenuResult = 23,

    OpenSellShop = 46,

    CheckSpecificGenericDialogIsOpen = 58,
    CheckSpecificPersonMenuIsOpen = 59,
}

impl TalkScriptExtender for TalkScript {
    fn clear_talk_list_data(&mut self) -> Result<(), EzStateInvokeError> {
        self.event(TalkScriptId::ClearTalkListData as i32)
    }

    fn add_talk_list_data(&mut self, index: i32, text_id: i32, unk1: i32) -> Result<(), EzStateInvokeError> {
        self.event((
            TalkScriptId::AddTalkListData as i32,
            [
                EzStateValue::Int32(index),
                EzStateValue::Int32(text_id),
                EzStateValue::Int32(unk1),
            ]
        ))
    }

    fn show_shop_message(&mut self, unk1: i32) -> Result<(), EzStateInvokeError> {
        self.event((
            TalkScriptId::ShowShopMessage as i32,
            [
                EzStateValue::Int32(unk1),
            ],
        ))
    }

    fn open_regular_shop(&mut self, start_id_inclusive: i32, end_id_inclusive: i32) -> Result<(), EzStateInvokeError> {
        self.event((
            TalkScriptId::OpenRegularShop as i32,
            [
                EzStateValue::Int32(start_id_inclusive),
                EzStateValue::Int32(end_id_inclusive)
            ],
        ))
    }

    fn open_sell_shop(&mut self, start_id_inclusive: i32, end_id_inclusive: i32) -> Result<(), EzStateInvokeError> {
        self.event((
            TalkScriptId::OpenSellShop as i32,
            [
                EzStateValue::Int32(start_id_inclusive),
                EzStateValue::Int32(end_id_inclusive)
            ],
        ))
    }

    fn get_talk_list_menu_result(&mut self) -> Result<EzStateValue, EzStateInvokeError> {
        self.env(TalkScriptId::GetTalkListMenuResult as i32)
    }

    fn check_specific_person_menu_is_open(&mut self, menu_type: MenuType, id: i32) -> Result<EzStateValue, EzStateInvokeError> {
        self.env((
            TalkScriptId::CheckSpecificPersonMenuIsOpen as i32,
            [
                EzStateValue::Int32(menu_type as i32),
                EzStateValue::Int32(id),
            ],
        ))
    }

    fn check_specific_person_generic_dialog_is_open(&mut self, id: i32) -> Result<EzStateValue, EzStateInvokeError> {
        self.env((
            TalkScriptId::CheckSpecificGenericDialogIsOpen as i32,
            [
                EzStateValue::Int32(id),
            ],
        ))
    }
}

