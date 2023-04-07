use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

pub(crate) mod dao;
pub(crate) mod dtos;
pub(crate) mod handlers;
pub(crate) mod models;
pub(crate) mod tower_services;

use crate::models::User;
use crate::tower_services::events::EventsAuthLayer;
use aws_config::meta::region::RegionProviderChain;
use aws_sdk_dynamodb::Client;
use axum::extract::ws::{Message, WebSocketUpgrade};
use axum::extract::State;
use axum::handler::Handler;
use axum::routing::{get, post};
use axum::Extension;
use axum::{extract::ws::WebSocket, response::Response, Router};
use chrono::Utc;
use futures::{
    sink::SinkExt,
    stream::{SplitSink, SplitStream, StreamExt},
};
use pnet::datalink;
use serde_json::Value;
use tokio::sync::broadcast::{Receiver, Sender};
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver};
use tokio::sync::Mutex;
use tower::ServiceBuilder;

#[tokio::main]
async fn main() {
    println!("hello world from events-service");
    let server_ip = datalink::interfaces()
        .into_iter()
        .filter(|iface| iface.name.contains("wl"))
        .next()
        .unwrap()
        .ips
        .into_iter()
        .filter(|ip| ip.is_ipv4())
        .next()
        .unwrap()
        .broadcast();
    println!("srver: {:?}", server_ip);
    // TODO: refactor this client instantiation logic.
    let client = {
        let region_provider = RegionProviderChain::default_provider().or_else("us-west-2");
        let config = aws_config::from_env()
            .region(region_provider)
            .endpoint_url("http://localhost:8000")
            .load()
            .await;

        Client::new(&config)
    };

    let event_app_state = dtos::EventsAppState {
        channels: Arc::new(Mutex::new(HashMap::new())),
        dynamodb: client,
        server_ip,
    };

    let router = Router::new().route(
        "/",
        get(
            handle_websocket_connection.layer(ServiceBuilder::new().layer(EventsAuthLayer {
                app_state: event_app_state.clone(),
            })),
        ),
    );

    let router = Router::new()
        .nest("/api/events", router)
        .route(
            "/api/events/send",
            post(crate::handlers::events::send_notification),
        )
        .with_state(event_app_state.clone());

    axum::Server::bind(&"127.0.0.1:3002".parse().unwrap())
        .serve(router.into_make_service())
        .await
        .unwrap()
}

#[axum_macros::debug_handler]
async fn handle_websocket_connection(
    ws: WebSocketUpgrade,
    State(events_app_state): State<dtos::EventsAppState>,
    Extension(user): Extension<User>,
) -> Response {
    ws.on_upgrade(|web_socket| handle_websocket_by_spltting(web_socket, user, events_app_state))
}

async fn handle_websocket_by_spltting(
    socket: WebSocket,
    user: User,
    events_app_state: dtos::EventsAppState,
) {
    let (sender, receiver) = socket.split();
    // let x: u64 = 2334;
    // Instant::now().elapsed()

    let (tx, rx) = unbounded_channel::<Value>();

    events_app_state
        .channels
        .lock()
        .await
        .insert(user.id.to_string(), tx);
    // events_app_state.server_ip.to_string
    /*
     * add user->server mapping in dynamo.
     * */

    let (close_signal_sender, close_signal_receiver) = tokio::sync::broadcast::channel::<()>(1);
    let mut close_signal_receiver2 = close_signal_sender.subscribe();
    // close_signal_receiver.clone();
    let task2 = tokio::spawn(handle_read(receiver, close_signal_sender));
    let task1 = tokio::spawn(handle_send(sender, close_signal_receiver, rx));
    let server_ip = events_app_state.server_ip;
    let state = events_app_state.clone();

    let task3 = tokio::spawn(async move {
        loop {
            // let sleep_task = tokio::time::sleep(Duration::from_secs(5));
            // let close_signal_task = close_signal_receiver2.recv();

            tokio::select! {
                _ = tokio::time::sleep(Duration::from_secs(5)) => {

                    let ping = models::ping::Ping { user_id:user.id.to_string(), valid_till: Utc::now(), server_ip: server_ip  };
                    dao::put_item(&state.dynamodb, &ping).await;
                    println!("pinged for user");
                },
                _ = close_signal_receiver2.recv() => {
                    break;
                }


            }
        }
    });

    let (_, _, _) = tokio::join!(task1, task2, task3);

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
    mut close_signal_receiver: Receiver<()>,
    mut event_rx: UnboundedReceiver<Value>,
) {
    // Wrapping the close_signal_receiver in tokio::spawn
    // mainly because don't want to loose signals inside "looped tokio::select".
    // Read more here. https://docs.rs/tokio/latest/tokio/task/struct.JoinHandle.html#cancel-safety
    let mut recieve_signal_handle = tokio::spawn(async move { close_signal_receiver.recv().await });

    loop {
        tokio::select! {
                msg = event_rx.recv() => {
                        println!("messages received from channel");
                let Some(msg) = msg else {
                    continue;
                };
        match sender
                        .send(Message::Text(msg.to_string()))
                        .await
                    {
                        Ok(_) => {
                            println!("message sent after receiving from channel");
                            // tokio::time::sleep(Duration::from_secs(2)).await;
                        }
                        Err(e) => {
                            eprintln!("got some error while sending, erro: {:?}", e);
                            break;
                        }
                    }

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
