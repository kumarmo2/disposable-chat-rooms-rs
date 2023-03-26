use crate::dtos::{events::SendEventNotificationRequest, EventsAppState};
use axum::{extract::State, Json};
use hyper::StatusCode;

pub(crate) async fn send_notification(
    State(events_app_state): State<EventsAppState>,
    Json(req): Json<SendEventNotificationRequest>,
) -> StatusCode {
    let user_id = &req.user_id;
    let guard = events_app_state.channels.lock().await;
    let Some(tx) =  guard.get(user_id) else {
        println!("could not find the hannel for the user: {}", user_id);
        return StatusCode::OK;
    };

    let tx = tx.clone();
    if let Err(e) = tx.send(req.payload) {
        println!("error while sending message to the channel");
    };
    StatusCode::OK
}
