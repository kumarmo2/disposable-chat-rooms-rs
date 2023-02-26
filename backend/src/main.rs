mod models;

use axum::{routing::get, Router};

#[tokio::main]
async fn main() {
    println!("Hello, world!");
    let router = Router::new().route("/", get(|| async { "hello world" }));

    axum::Server::bind(&"127.0.0.1:3001".parse().unwrap())
        .serve(router.into_make_service())
        .await
        .unwrap()

    // print!("dfsd")
}
