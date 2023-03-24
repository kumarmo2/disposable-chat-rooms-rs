use axum::{extract::State, Extension, Json};
use hyper::StatusCode;

use crate::{
    dao,
    dtos::{message::CreateMessageRequest, ApiResult, AppState},
    models::{member::Member, message::Messsage, Room, User},
};

pub(crate) async fn create_message(
    Extension(user): Extension<User>,
    State(app_state): State<AppState>,
    Json(req): Json<CreateMessageRequest>,
) -> Result<
    Json<ApiResult<String, &'static str>>,
    (StatusCode, Json<ApiResult<String, &'static str>>),
> {
    let member_partition_key = Member::get_partition_key_from_room_id(&req.room_id);
    let member_sort_key = Member::get_sort_key_from_user_id(&user.id);

    // TODO: check if room exists?
    let res = match dao::get_item_by_primary_key::<Member>(
        &app_state.dynamodb,
        &member_partition_key,
        Some(&member_sort_key),
    )
    .await
    {
        Ok(res) => res,
        Err(e) => {
            println!("got error while querying, err: {:?}", e);
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResult::Error("Something went wrong, Please try again!")),
            ));
        }
    };

    let Some(member) = res else{
        return Ok(Json(ApiResult::Error("user is not the member of the room")));
    };

    let message = Messsage::from_fields(
        rusty_ulid::generate_ulid_string(),
        req.room_id.to_string(),
        req.content,
        user.id,
        member.display_name,
    );

    if let Err(e) = dao::put_item(&app_state.dynamodb, &message).await {
        println!("error while creating message, err: {:?}", e);
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResult::Error("Something went wrong, Please try again!")),
        ));
    }
    // TODO: publish the message event.

    Ok(Json(ApiResult::Result(message.id)))
}
