use std::time::Duration;

use axum::extract::ws::{Message, WebSocketUpgrade};
use axum::routing::get;
use axum::{extract::ws::WebSocket, response::Response, Router};
use futures::{
    sink::SinkExt,
    stream::{SplitSink, SplitStream, StreamExt},
};
use tokio::sync::oneshot::{Receiver, Sender};

#[tokio::main]
async fn main() {
    println!("hello world from events-service");

    let router = Router::new().route("/", get(handle_websocket_connection));

    let router = Router::new().nest("/api/events", router);

    axum::Server::bind(&"127.0.0.1:3002".parse().unwrap())
        .serve(router.into_make_service())
        .await
        .unwrap()
}

async fn handle_websocket_connection(ws: WebSocketUpgrade) -> Response {
    ws.on_upgrade(handle_websocket_by_spltting)
}

async fn handle_websocket_by_spltting(socket: WebSocket) {
    let (sender, receiver) = socket.split();

    // let (close_signal_sender, close_signal_receiver) = tokio::sync::oneshot::channel::<()>();
    let task2 = tokio::spawn(handle_read(receiver));
    let task1 = tokio::spawn(handle_send(sender));

    tokio::join!(task1, task2);

    // futures::join!(task1, task2);
}

async fn handle_read(mut receiver: SplitStream<WebSocket>) {
    while let Some(msg_res) = receiver.next().await {
        let Ok(msg) = msg_res else {
            println!("error while receing msg");
            // TODO: remove use of unwrap
            // close_signal_sender.send(()).unwrap();
            break;
        };

        if let Message::Close(_) = msg {
            // TODO: remove use of unwrap
            println!("got close signal from client");
            // close_signal_sender.send(()).unwrap();
            break;
        }
    }
    println!("returning from handle_read");
}

async fn handle_send(mut sender: SplitSink<WebSocket, Message>) {
    loop {
        match sender
            .send(Message::Text("some message -v2".to_string()))
            .await
        {
            Ok(_) => {
                println!("message sent, sleeping now");
                tokio::time::sleep(Duration::from_secs(2)).await;
            }
            Err(e) => {
                eprintln!("got some error while sending, erro: {:?}", e);
                break;
            }
        };
    }
    println!("returning from handle_send");
}
