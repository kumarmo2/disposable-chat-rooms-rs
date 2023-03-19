// #![warn()]
use serde::Deserialize;

#[derive(Deserialize)]
pub(crate) struct CreateMessageRequest {
    pub(crate) room_id: String,
    pub(crate) content: String,
}
