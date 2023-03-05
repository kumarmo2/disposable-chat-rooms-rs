#![feature(async_fn_in_trait)]
#![feature(allocator_api)]
mod models;
mod tower_services;

use axum::http::StatusCode;

use axum::middleware::from_extractor;
use axum::Json;
use axum::{routing::get, Router};
use axum_extra::extract::CookieJar;
use models::extractors::UserExtractor;
use serde_json::{json, Value};
use tower::ServiceBuilder;

use crate::tower_services::UserLayer;

#[tokio::main]
async fn main() {
    println!("sdfsdfl");
    let router = Router::new()
        .route("/", get(home_handler))
        .route("/cookie", get(route_with_cookie))
        .layer(ServiceBuilder::new().layer(UserLayer {}));

    axum::Server::bind(&"127.0.0.1:3001".parse().unwrap())
        .serve(router.into_make_service())
        .await
        .unwrap()
}

#[axum_macros::debug_handler]
async fn home_handler(UserExtractor(user): UserExtractor) -> Json<Value> {
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
