// #![warn()]
use serde::{Deserialize, Serialize};

use crate::models::message::Messsage;

#[derive(Deserialize)]
pub(crate) struct CreateMessageRequest {
    pub(crate) room_id: String,
    pub(crate) content: String,
}

#[derive(Serialize)]
pub(crate) struct MessageDto {
    pub(crate) content: String,
    pub(crate) id: String,
    pub(crate) sent_by_user_id: String,
    pub(crate) sent_by_member_name: String,
}

impl From<Messsage> for MessageDto {
    fn from(value: Messsage) -> Self {
        Self {
            content: value.content,
            id: value.id,
            sent_by_user_id: value.sent_by_user_id,
            sent_by_member_name: value.sent_by_member_name,
        }
    }
}

#[derive(Serialize)]
pub(crate) struct GetMessagesResponse {
    pub(crate) messages: Vec<MessageDto>,
}
