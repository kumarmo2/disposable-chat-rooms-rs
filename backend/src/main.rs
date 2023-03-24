#![warn(dead_code)]
pub(crate) mod dao;
mod dtos;
mod handlers;
mod models;
mod tower_services;

use axum::error_handling::HandleErrorLayer;
use axum::response::{IntoResponse, Response};
use axum::routing::post;
use dtos::State;
use std::sync::Arc;

use aws_config::meta::region::RegionProviderChain;
use aws_sdk_dynamodb::Client;
use axum::http::StatusCode;

use axum::{routing::get, Router};
use axum::{Extension, Json};
use axum_extra::extract::CookieJar;
use handlers::{create_room, get_members_in_room, get_messages, join_room};
use models::User;
use serde_json::{json, Value};
use tower::ServiceBuilder;

use crate::tower_services::UserLayer;

#[tokio::main]
async fn main() {
    let client = {
        let region_provider = RegionProviderChain::default_provider().or_else("us-west-2");
        let config = aws_config::from_env()
            .region(region_provider)
            .endpoint_url("http://localhost:8000")
            .load()
            .await;

        Client::new(&config)
    };

    let app_state = Arc::new(State { dynamodb: client });

    let room_routes = Router::new()
        .route("/", post(create_room))
        .route("/:room_id/join", post(join_room))
        .route("/:room_id/messages", get(get_messages))
        .route("/:room_id/members", get(get_members_in_room));

    let room_routes = Router::new().nest("/rooms", room_routes);

    let message_routes = Router::new().route("/", post(handlers::message::create_message));

    let message_routes = Router::new().nest("/messages", message_routes);

    let apis = Router::new()
        .route("/", get(home_handler))
        .route("/cookie", get(route_with_cookie));

    let apis = Router::new()
        .merge(room_routes)
        .merge(apis)
        .merge(message_routes);

    let router = Router::new()
        .nest("/api", apis)
        .with_state(app_state.clone())
        .layer(
            ServiceBuilder::new()
                .layer(HandleErrorLayer::new(handle_service_error))
                .layer(UserLayer(app_state.clone())),
        );

    axum::Server::bind(&"127.0.0.1:3001".parse().unwrap())
        .serve(router.into_make_service())
        .await
        .unwrap()
}

async fn handle_service_error<E>(err: crate::tower_services::Error<E>) -> Response
where
    E: IntoResponse,
{
    err.into_response()
}

#[axum_macros::debug_handler]
async fn home_handler(Extension(user): Extension<User>) -> Json<Value> {
    println!("home handler, user: {:?}", user);
    Json(json!({ "result": "Hello, world"}))
}

#[axum_macros::debug_handler]
async fn route_with_cookie(
    cookies: CookieJar,
) -> Result<(), (StatusCode, Json<serde_json::Value>)> {
    let Some(user_cookie) = cookies.get("user") else {
        return Err((StatusCode::UNAUTHORIZED, Json(json!({ "error": "no user cookie found"}))));
    };
    Ok(())
}
