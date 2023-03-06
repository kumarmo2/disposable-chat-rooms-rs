#![feature(async_fn_in_trait)]
#![feature(allocator_api)]
#![warn(dead_code)]
mod dao;
mod models;
mod tower_services;

use std::sync::Arc;
use tokio::sync::Mutex;

use aws_config::meta::region::RegionProviderChain;
use aws_sdk_dynamodb::Client;
use axum::http::StatusCode;

use axum::{routing::get, Router};
use axum::{Extension, Json};
use axum_extra::extract::CookieJar;
use models::User;
use serde_json::{json, Value};
use tower::ServiceBuilder;

use crate::tower_services::UserLayer;

#[derive(Clone)]
pub(crate) struct State {
    pub(crate) dynamodb: Client,
}

type AppState = Arc<Mutex<State>>;

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

    let app_state = Arc::new(Mutex::new(State { dynamodb: client }));

    let router = Router::new()
        .route("/", get(home_handler))
        .route("/cookie", get(route_with_cookie))
        .layer(ServiceBuilder::new().layer(UserLayer(app_state.clone())));

    axum::Server::bind(&"127.0.0.1:3001".parse().unwrap())
        .serve(router.into_make_service())
        .await
        .unwrap()
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
