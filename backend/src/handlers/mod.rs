// use std::future;

pub(crate) mod message;
use axum::{
    extract::{Path, State},
    Extension, Json,
};
use hyper::StatusCode;

use crate::{
    dao::{self, room::get_room_by_id},
    dtos::{
        message::{GetMessagesResponse, MessageDto},
        ApiResult, AppState, CreateRoomRequest, JoinRoomRequest,
    },
    models::{member::Member, Room, User},
};

pub(crate) async fn get_messages(
    Path(room_id): Path<String>,
    State(app_state): State<AppState>,
    Extension(_): Extension<User>,
) -> Result<
    Json<ApiResult<GetMessagesResponse, &'static str>>,
    (StatusCode, Json<ApiResult<&'static str, &'static str>>),
> {
    // TODO: add pagination for messages.

    println!("room_id: {}", room_id);
    let room_details_by_id_fut = dao::room::get_room_by_id(&app_state.dynamodb, &room_id);

    let messages_fut = dao::message::get_messages(&room_id, &app_state.dynamodb);

    let (room_details, messages) = tokio::join!(room_details_by_id_fut, messages_fut);

    if let None = room_details {
        return Err((
            StatusCode::OK,
            Json(ApiResult::Error("room doesn't exists")),
        ));
    }

    if let Err(e) = messages {
        println!("Err while getting messages, error: {:?}", e);
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResult::Error("Something went wrong. Please try again!")),
        ));
    }
    let messages = messages.unwrap();
    println!("messages.length: {}", messages.len());
    let response = GetMessagesResponse {
        messages: messages.into_iter().map(|m| MessageDto::from(m)).collect(),
    };
    Ok(Json(ApiResult::Result(response)))
}

pub(crate) async fn join_room(
    Path(room_id): Path<String>,
    Extension(user): Extension<User>,
    State(app_state): State<AppState>,
    Json(request): Json<JoinRoomRequest>,
) -> Result<StatusCode, (StatusCode, Json<ApiResult<&'static str, &'static str>>)> {
    /*
     * - check if room exists, if no, return error.
     * - check if user is already a member of the room, if yes, return error.
     * - create member entry.
     *
     * */

    let room_details_future = get_room_by_id(&app_state.dynamodb, &room_id);

    let member_partition_key = Member::get_partition_key_from_room_id(&room_id);

    let member_sort_key = Member::get_sort_key_from_user_id(&user.id);

    let member_details_fut = dao::get_item_by_primary_key::<Member>(
        &app_state.dynamodb,
        &member_partition_key,
        Some(&member_sort_key),
    );

    let (room_details, member_details): (Option<_>, Result<_, _>) =
        futures::join!(room_details_future, member_details_fut);

    let Some(_) = room_details else {
        return Err((StatusCode::OK, Json(ApiResult::Error("room doesn't exist"))));
    };

    let Ok(result) = member_details else {
        return Err((StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResult::Error("Something went wrong. Please try!"))));
    };

    let None = result else {
        return Err((StatusCode::OK, Json(ApiResult::Error("User is already a member"))));
    };

    let member_item = Member::from_fields(request.display_name, room_id, user.id);

    dao::put_item(&app_state.dynamodb, &member_item)
        .await
        .and_then(|_| Ok(StatusCode::OK))
        .or_else(|e| {
            println!("error while inserting member: {:?}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResult::Error(
                    "Something went wrong. Please try again!sdfsdf",
                )),
            ))
        })

    // insert the member.

    // todo!()
}

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
