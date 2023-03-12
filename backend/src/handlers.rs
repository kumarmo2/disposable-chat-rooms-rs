// use std::future;

use axum::{
    extract::{Path, State},
    Extension, Json,
};
use hyper::StatusCode;

use crate::{
    dao::{self, room::get_room_by_id},
    dtos::{ApiResult, CreateRoomRequest},
    models::{member::Member, Room, User},
    AppState,
};

#[axum_macros::debug_handler]
pub(crate) async fn get_members_in_room(
    Path(room_id): Path<String>,
    // Extension(logged_in_user): Extension<User>,
    State(app_state): State<AppState>,
) -> Result<
    Json<ApiResult<Vec<Member>, &'static str>>,
    (StatusCode, Json<ApiResult<Vec<Member>, &'static str>>),
> {
    let room_details_future = dao::room::get_room_by_id(&app_state.dynamodb, &room_id);
    let room_members_future = dao::room::get_rooom_members(&app_state.dynamodb, &room_id);

    let (room_details, room_members): (_, Result<Vec<Member>, _>) =
        futures::join!(room_details_future, room_members_future);

    let Some(_) = room_details else {
        return Ok(Json(ApiResult::Error("Room doesn't exists")));
    };

    room_members
        .and_then(|members| Ok(Json(ApiResult::Result(members))))
        .or_else(|e| {
            print!("err while getting members: {:?}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResult::Error("something went wrong. Please try again!")),
            ))
        })
}

#[axum_macros::debug_handler]
pub(crate) async fn create_room(
    Extension(user): Extension<User>,
    State(app_state): State<AppState>,
    Json(req): Json<CreateRoomRequest>,
) -> Result<
    Json<ApiResult<String, &'static str>>,
    (StatusCode, Json<ApiResult<String, &'static str>>),
> {
    let room_option = get_room_by_id(&app_state.dynamodb, req.id.as_ref()).await;

    if let Some(_) = room_option {
        return Err((
            StatusCode::OK,
            Json(ApiResult::<String, &str>::Error(
                "Room with the same id already exists",
            )),
        ));
    }

    let new_room =
        Room::from_fields::<String>(req.id, req.display_name, user.id.as_str().to_string());
    if let Err(e) = dao::put_item(&app_state.dynamodb, &new_room)
        .await
        .or_else(|err| {
            println!("error while inserting: {:?}", err);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResult::<String, &str>::Error(
                    "Something went wrong, Please try again later!",
                )),
            ))
        })
    {
        return Err(e);
    }

    //Create member entry.
    let member = Member::from_fields(
        req.member_name,
        new_room.id().to_string(),
        user.id.as_str().to_string(),
    );
    if let Err(e) = dao::put_item(&app_state.dynamodb, &member)
        .await
        .or_else(|e| {
            println!("error while inserting member: {:?}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResult::<String, &str>::Error(
                    "Something went wrong, Please try again later!",
                )),
            ))
        })
    {
        return Err(e);
    };

    Ok(Json(ApiResult::Result(new_room.id)))
}
