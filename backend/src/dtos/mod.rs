pub(crate) mod message;

use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};
use tokio::sync::Mutex;

use aws_sdk_dynamodb::{
    error::{PutItemError, QueryError},
    types::SdkError,
    Client,
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

#[derive(Debug)]
pub(crate) enum DaoError {
    Internal(String),
    PutError(SdkError<PutItemError>),
    QueryError(SdkError<QueryError>),
}

#[derive(Clone)]
pub(crate) struct State {
    pub(crate) dynamodb: Client,
}

#[derive(Clone)]
pub(crate) struct EventsAppState {
    pub(crate) channels: Arc<Mutex<HashMap<String, UnboundedSender<Value>>>>,
}

pub(crate) type AppState = Arc<State>;
