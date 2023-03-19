use std::time::Duration;

use axum::extract::ws::{Message, WebSocketUpgrade};
use axum::routing::get;
use axum::{extract::ws::WebSocket, response::Response, Router};
use futures::{
    sink::SinkExt,
    stream::{SplitSink, SplitStream, StreamExt},
};
use tokio::sync::oneshot::{Receiver, Sender};
// use tokio::sync::watch::Sender;

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

    let (close_signal_sender, close_signal_receiver) = tokio::sync::oneshot::channel::<()>();
    let task2 = tokio::spawn(handle_read(receiver, close_signal_sender));
    let task1 = tokio::spawn(handle_send(sender, close_signal_receiver));

    tokio::join!(task1, task2);

    // futures::join!(task1, task2);
}

async fn handle_read(mut receiver: SplitStream<WebSocket>, close_signal_sender: Sender<()>) {
    while let Some(msg_res) = receiver.next().await {
        let Ok(msg) = msg_res else {
            println!("error while receing msg");
            // TODO: remove use of unwrap
            close_signal_sender.send(()).unwrap();
            break;
        };

        if let Message::Close(_) = msg {
            // TODO: remove use of unwrap
            println!("got close signal from client");
            close_signal_sender.send(()).unwrap();
            break;
        }
    }
    println!("returning from handle_read");
}

async fn handle_send(
    mut sender: SplitSink<WebSocket, Message>,
    close_signal_receiver: Receiver<()>,
) {
    // Wrapping the close_signal_receiver in tokio::spawn
    // mainly because don't want to loose signals inside "looped tokio::select".
    // Read more here. https://docs.rs/tokio/latest/tokio/task/struct.JoinHandle.html#cancel-safety
    let mut recieve_signal_handle = tokio::spawn(async move { close_signal_receiver.await });

    loop {
        tokio::select! {
            _ = tokio::time::sleep(Duration::from_secs(2)) => {
            // we are here that we received a message from message broke/internal messaging system.


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
        },
        _ = &mut recieve_signal_handle => {
            println!("receing close signal inside select");
                return;
        }
        }
    }

    // todo!();

    // println!("returning from handle_send");
}
