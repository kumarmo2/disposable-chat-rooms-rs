use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Deserialize)]
pub(crate) struct SendEventNotificationRequest {
    pub(crate) user_id: String,
    pub(crate) payload: Value,
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct MessageEvent<'a> {
    pub(crate) room_id: &'a str,
    pub(crate) message_id: &'a str,
}
