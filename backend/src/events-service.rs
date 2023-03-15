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
    // ws.on_upgrade(handle_websocket)
    ws.on_upgrade(handle_websocket_by_spltting)
}

async fn handle_websocket_by_spltting(socket: WebSocket) {
    let (mut sender, receiver) = socket.split();

    let (close_signal_sender, close_signal_receiver) = tokio::sync::oneshot::channel::<()>();
    let task2 = tokio::spawn(handle_read(receiver, close_signal_sender));
    let task1 = tokio::spawn(handle_send(sender, close_signal_receiver));
    futures::join!(task1, task2);
}

async fn handle_read(mut receiver: SplitStream<WebSocket>, close_signal_sender: Sender<()>) {
    while let Some(msg_res) = receiver.next().await {
        let Ok(msg) = msg_res else {
            println!("error while receing msg");
            // TODO: remove use of unwrap
            close_signal_sender.send(()).unwrap();
            return;
        };

        if let Message::Close(_) = msg {
            // TODO: remove use of unwrap
            println!("got close signal from client");
            close_signal_sender.send(()).unwrap();
            return;
        }

        println!("got msg of some other type. ignoring that");
    }

    // loop {
    // println!("waiting to recieve message...");
    // let found_close: Result<bool, ()> = match receiver.next().await {
    // Some(msg) => msg
    // .and_then(|m| match m {
    // Message::Close(_) => {
    // println!("got close message");
    // close_signal_sender.send(());
    // Ok(true)
    // }
    // _ => {
    // println!("got message which is not close");
    // Ok(false)
    // }
    // })
    // .or_else(|_| Ok(true)),
    // None => {
    // println!("None variant received");
    // Ok(true)
    // }
    // };
    // if found_close.unwrap() {
    // println!("returning from here.....");
    // return;
    // }
    // }
}

async fn handle_send(
    mut sender: SplitSink<WebSocket, Message>,
    close_signal_receiver: Receiver<()>,
) {
    // loop {
    // futures::select! {
    // _ = close_signal_receiver => ()

    // };
    // }

    loop {
        match sender
            .send(Message::Text("some message -v2".to_string()))
            .await
        {
            Ok(_) => {
                println!("message sent");

                tokio::time::sleep(Duration::from_secs(2)).await;
            }
            Err(e) => {
                eprintln!("got error while sending, error: {:?}", e);
                return;
            }
        }
    }
}

async fn handle_websocket(mut socket: WebSocket) {
    // socket.re
    // let (mut sender, mut receiver): (SplitSink<_, _>, SplitStream<_>) = socket.split();
    loop {
        let found_error = {
            match socket
                .send(Message::Text("some messsage".to_string()))
                .await
            {
                Ok(_) => {
                    println!("message sent");
                    false
                }
                Err(e) => {
                    eprint!("error while sending message,e: {:?}", e);
                    true
                }
            }
        };
        if found_error {
            match socket.close().await {
                Ok(_) => println!("websocket closed"),
                Err(e) => eprintln!("could not close the socket, error: {:?}", e),
            }
            return;
        }
        tokio::time::sleep(Duration::from_secs(2)).await;
    }
}

// async fn handle_recieve_end(receiver: SplitStream<WebSocket>) {

// }
