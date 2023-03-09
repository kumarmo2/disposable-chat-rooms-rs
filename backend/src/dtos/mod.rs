use serde::Deserialize;

#[derive(Deserialize)]
pub(crate) struct CreateRoomRequest {
    pub(crate) id: String, /*ULID*/
    pub(crate) display_name: String,
}
