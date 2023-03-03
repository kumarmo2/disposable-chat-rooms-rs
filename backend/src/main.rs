mod models;

use axum::http::{Request, StatusCode};
use axum::middleware::from_fn;
use axum::middleware::Next;
use axum::Json;
use axum::{routing::get, Router};
use axum_extra::extract::cookie::{Cookie, CookieJar};
use rusty_ulid::{generate_ulid_string, Ulid};
use serde_json::json;

#[tokio::main]
async fn main() {
    println!("sdfsdfl");
    let router = Router::new()
        .route("/", get(|| async { "hello world" }))
        .route("/cookie", get(route_with_cookie))
        .layer(from_fn(user_cookie_middleware));

    axum::Server::bind(&"127.0.0.1:3001".parse().unwrap())
        .serve(router.into_make_service())
        .await
        .unwrap()
}

async fn user_cookie_middleware<B>(
    mut cookies: CookieJar,
    request: Request<B>,
    next: Next<B>,
) -> (CookieJar, axum::response::Response) {
    if let None = cookies.get("user") {
        // This cookie won't be set until jar is returned in response.
        cookies = cookies.add(Cookie::new("user", generate_ulid_string()));
    }
    let response = next.run(request).await;
    (cookies, response)
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
