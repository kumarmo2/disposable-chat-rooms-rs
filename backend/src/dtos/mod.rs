use aws_sdk_dynamodb::{
    error::{PutItemError, QueryError},
    types::SdkError,
};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub(crate) struct CreateRoomRequest {
    pub(crate) id: String, /*ULID*/
    pub(crate) display_name: String,
    pub(crate) member_name: String,
}

#[derive(Deserialize)]
pub(crate) struct JoinRoomRequest {
    pub(crate) display_name: String,
}

#[derive(Serialize)]
pub(crate) enum ApiResult<T, E>
where
    T: Serialize,
    E: Serialize,
{
    Result(T),
    Error(E),
}

pub(crate) enum DaoError {
    Internal(String),
    PutError(SdkError<PutItemError>),
    QueryError(SdkError<QueryError>),
}
