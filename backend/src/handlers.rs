use axum::{extract::State, Extension, Json};
use hyper::StatusCode;
use serde_json::{json, Value};

use crate::{
    dao,
    dtos::CreateRoomRequest,
    models::{Room, User},
    AppState,
};

#[axum_macros::debug_handler]
pub(crate) async fn create_room(
    Extension(user): Extension<User>,
    State(app_state): State<AppState>,
    Json(req): Json<CreateRoomRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let guard = app_state.lock().await;
    let room_option = dao::get_room_by_id(&guard.dynamodb, req.id.as_ref()).await;

    if let Some(_) = room_option {
        return Err((
            StatusCode::OK,
            Json(json!({ "err": "Room with the same id already exists"})),
        ));
    }

    let new_room = Room::from_fields::<String>(req.id, req.display_name, user.id);
    dao::put_item(&guard.dynamodb, &new_room)
        .await
        .and_then(|_| Ok(Json(json!({ "result" : &new_room.id}))))
        .or_else(|err| {
            println!("error while inserting: {:?}", err);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error" : "Something went wrong, Please try again later!"})),
            ))
        })
}
