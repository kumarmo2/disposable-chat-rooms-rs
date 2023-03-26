use serde::Deserialize;
use serde_json::Value;

#[derive(Deserialize)]
pub(crate) struct SendEventNotificationRequest {
    pub(crate) user_id: String,
    pub(crate) payload: Value,
}
