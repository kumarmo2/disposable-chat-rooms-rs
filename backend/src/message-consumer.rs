mod dao;
mod dtos;
mod models;
mod utils;
use std::net::IpAddr;

use aws_config::meta::region::RegionProviderChain;
use aws_sdk_dynamodb::Client;
use futures::stream::StreamExt;
use lapin::{self, message::Delivery, options::*, types::FieldTable, Channel};
use pnet::datalink;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
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

async fn handle_delivery(delivery: Delivery, dynamodb: Client) -> anyhow::Result<()> {
    let data = &delivery.data;
    let message_event = serde_json::from_slice::<dtos::events::MessageEvent>(&data).unwrap();
    println!("data: {:?}", message_event);
    let message_fut = dao::get_item_by_primary_key::<models::message::Messsage, &str, &str>(
        &dynamodb,
        &message_event.room_id,
        Some(&message_event.message_id),
    );

    let members_fut = dao::room::get_rooom_members(&dynamodb, &message_event.room_id);
    let (message_result, members_result) = tokio::join!(message_fut, members_fut);
    let (message, members) = match (message_result, members_result) {
        (Err(e), _) => {
            println!("error fetching message, err: {:?}", e);
            return Ok(());
        }
        (_, Err(e)) => {
            println!("error retrieving members, err: {:?}", e);
            return Ok(());
        }
        (Ok(None), _) => {
            println!("found no message for the event, {:?}", message_event);
            acknowledge_deliver(delivery).await;
            return Ok(());
        }
        (Ok(Some(message)), Ok(members)) => (message, members),
    };
    let filtered_members = members
        .iter()
        .filter(|members| members.user_id != message.sent_by_user_id);
    /*
     * TODO
     * Get  the ip addresses of the events server, to which these members are connected.
     * Call the endpoints for each member to deliver these messages to them.
     * */

    acknowledge_deliver(delivery).await;
    return Ok(());
}
