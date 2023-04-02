mod dao;
mod dtos;
mod models;
mod utils;
use aws_config::meta::region::RegionProviderChain;
use aws_sdk_dynamodb::Client;
use futures::stream::StreamExt;
use lapin::{self, message::Delivery, options::*, types::FieldTable, Channel};

#[tokio::main]
async fn main() -> lapin::Result<()> {
    println!("hello from message consumer");
    let connection = crate::utils::rabbitmq::create_connection().await?;
    let channel: Channel = connection.create_channel().await?;

    let client = {
        let region_provider = RegionProviderChain::default_provider().or_else("us-west-2");
        let config = aws_config::from_env()
            .region(region_provider)
            .endpoint_url("http://localhost:8000")
            .load()
            .await;

        Client::new(&config)
    };
    let mut consumer = channel
        .basic_consume(
            "message-queue",
            "",
            BasicConsumeOptions::default(),
            FieldTable::default(),
        )
        .await?;

    // TODO: need to check how to recieve multiple messages at once.
    while let Some(delivery_result) = consumer.next().await {
        match delivery_result {
            Err(err) => println!("err while receving the message, err: {:?}", err),
            Ok(delivery) => {
                let client = client.clone();
                tokio::task::spawn(async move { handle_delivery(delivery, client).await });
            }
        }
    }

    // channel.queue_bind();

    Ok(())
}

async fn acknowledge_deliver(delivery: Delivery) {
    delivery
        .ack(BasicAckOptions::default())
        .await
        .unwrap_or_else(|e| {
            println!("error while acknowleding message, err: {:?}", e);
        });
}

async fn handle_delivery(delivery: Delivery, dynamodb: Client) {
    let data = &delivery.data;
    let message = serde_json::from_slice::<dtos::events::MessageEvent>(&data).unwrap();
    println!("data: {:?}", message);
    // let members = match dao::room::get_rooom_members(&dynamodb, message.room_id).await {
    // Err(e) => {
    // println!("error while fetching members, err: {:?}", e);
    // return;
    // }
    // Ok(members) => members,
    // };

    // if members.len() == 0 {
    // acknowledge_deliver(delivery).await;
    // return;
    // }
    // members.iter().filter(|member| member.user_id != message.)
    // if let Err(e) = dao::room::get_rooom_members(&dynamodb, message.room_id).await {};

    // TODO: from the message, get the members of the room, and send the
    // "message" type notification to the members except for the sender of the
    // message.
    acknowledge_deliver(delivery).await;
}
