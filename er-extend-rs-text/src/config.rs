use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Deserialize, Default)]
pub struct ExtraText {
    pub event_text_for_talk: Vec<EventTextForTalk>,
}

#[derive(Debug, Deserialize, Default)]
pub struct EventTextForTalk {
    pub text_id: i32,
    pub text: String,
}

enum TextCategory {
    EventTextForTalk = 33,
}

impl ExtraText {
    pub fn new(event_text_for_talk_map: Vec<EventTextForTalk>) -> Self {
        ExtraText {
            event_text_for_talk: event_text_for_talk_map,
        }
    }

    pub fn has_text_overrides(&self) -> bool {
        !self.event_text_for_talk.is_empty()
    }

    pub fn generate_overridden_messages(&self) -> HashMap<u32, HashMap<i32, Vec<u16>>> {
        let mut overridden_messages = HashMap::new();

        let mut event_text_for_talk_map: HashMap<i32, Vec<u16>> = HashMap::new();
        for event_text_entry in self.event_text_for_talk.iter() {
            let text = {
                let mut text: Vec<u16> = event_text_entry.text.encode_utf16().collect();
                text.push(0);
                text
            };
            event_text_for_talk_map.insert(event_text_entry.text_id, text);
        }

        overridden_messages.insert(TextCategory::EventTextForTalk as u32, event_text_for_talk_map);
        overridden_messages
    }
}
