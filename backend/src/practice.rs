// use aws::config::meta::

#![allow(dead_code)]
#![allow(unused_imports)]

use aws_config::meta::region::RegionProviderChain;
use aws_sdk_dynamodb::{
    model::{
        AttributeDefinition, AttributeValue, KeySchemaElement, ProvisionedThroughput,
        ScalarAttributeType,
    },
    Client, Error,
};
use rusty_ulid::{generate_ulid_string, Ulid};
use serde_json::{json, Value};
use std::time::{Instant, SystemTime};

mod models;
use models::Room;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let ulid = generate_ulid_string();
    println!("ulid: {}", ulid);

    let ulid2 = Ulid::generate();
    // std::error::Error

    println!("ulid2: {:?}", ulid2);

    let region_provider = RegionProviderChain::default_provider().or_else("us-west-2");
    let config = aws_config::from_env()
        .region(region_provider)
        .endpoint_url("http://localhost:8000")
        .load()
        .await;

    let client = Client::new(&config);
    // list_tables(&client).await?;

    // add_room(&client, Room::new("room-1")).await?;
    scan_main_table(&client).await?;
    Ok(())
}

async fn scan_main_table(client: &Client) -> Result<(), Error> {
    let output = client.scan().table_name("main").send().await?;

    println!("items count: {}", output.count());

    let Some(items) = output.items() else {
        return Ok(());
    };

    for item in items {
        println!("key: {:?}", item.get_key_value("pk").unwrap());

        let id = item.get("id").unwrap().as_s().unwrap();
        let name = item.get("name").unwrap().as_s().unwrap();

        let json: Value = json!({
            "id": id,
            "name": name
        });

        // let room = Room::from_fields(id, name);
        println!("room_json: {:#?}", json);
    }

    Ok(())
}

async fn add_room(client: &Client, room: Room) -> Result<(), Error> {
    let attrs = [
        ("pk", AttributeValue::S(room.get_partition_key())),
        ("sk", AttributeValue::S(room.get_sort_key())),
        ("name", AttributeValue::S(room.name().to_owned())),
        ("id", AttributeValue::S(room.id().to_owned())),
    ];

    let mut put_item_request = client.put_item().table_name("main");
    for (k, v) in attrs {
        put_item_request = put_item_request.item(k, v);
    }

    let _ = put_item_request.send().await?;
    println!("successfully inserted");

    Ok(())
}

async fn query_one_item(client: &Client) -> Result<(), Error> {
    // client.query().table_name("main").key_condition_expression("# = :")
    todo!()
}

async fn list_tables(client: &Client) -> Result<(), Error> {
    let list_tables_output = client.list_tables().send().await?;
    let table_names = list_tables_output.table_names().unwrap();

    for table in table_names.iter() {
        println!("table: {}", table);
    }

    Ok(())
}

async fn create_main_table(client: &Client) -> Result<(), Error> {
    // KeySchemaElement::attribute_name

    let partition_key_attr = AttributeDefinition::builder()
        .attribute_name("pk")
        .attribute_type(ScalarAttributeType::S)
        .build();

    let sorting_key_attr = AttributeDefinition::builder()
        .attribute_name("sk")
        .attribute_type(ScalarAttributeType::S)
        .build();

    let pt = ProvisionedThroughput::builder()
        .read_capacity_units(10)
        .write_capacity_units(5)
        .build();

    let create_table_output = client
        .create_table()
        .table_name("main")
        .provisioned_throughput(pt)
        .attribute_definitions(partition_key_attr)
        .attribute_definitions(sorting_key_attr)
        .key_schema(
            KeySchemaElement::builder()
                .attribute_name("pk")
                .key_type(aws_sdk_dynamodb::model::KeyType::Hash)
                .build(),
        )
        .key_schema(
            KeySchemaElement::builder()
                .attribute_name("sk")
                .key_type(aws_sdk_dynamodb::model::KeyType::Range)
                .build(),
        )
        .send()
        .await?;

    Ok(())
}

fn generate_nano_seconds_from_epoch() {
    let nano = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_nanos();

    println!("nanosecs: {}", nano);
}

async fn setup_dynamo() -> Result<(), Error> {
    println!("hello from practice");

    let region_provider = RegionProviderChain::default_provider().or_else("us-west-2");
    let config = aws_config::from_env()
        .region(region_provider)
        .endpoint_url("http://localhost:8000")
        .load()
        .await;

    let client = Client::new(&config);

    let resp = client.list_tables().send().await?;

    let Some(tables) = resp.table_names() else {
        println!("no tables found");
        return Ok(());
    };

    println!("table length: {}", tables.len());

    Ok(())
}
