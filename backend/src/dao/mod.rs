use std::collections::HashMap;

use aws_sdk_dynamodb::{
    error::PutItemError, model::AttributeValue, output, types::SdkError, Client,
};
use serde_dynamo::from_item;

use crate::models::Room;

const MAIN_TABLE_NAME: &'static str = "main";

pub(crate) type BoxedAttributes = Box<dyn Iterator<Item = (&'static str, AttributeValue)>>;

pub(crate) trait DynamoItem {
    // NOTE: attributes shouldn't send the "pk" and "sk". For them, we have another trait methods.
    fn attributes(&self) -> BoxedAttributes;
    fn pk(&self) -> String;
    fn sk(&self) -> Option<String>;
}

pub(crate) async fn put_item<T>(client: &Client, item: &T) -> Result<(), SdkError<PutItemError>>
where
    T: DynamoItem,
{
    let mut put_item_request = client.put_item().table_name("main");

    for (key, value) in item.attributes() {
        put_item_request = put_item_request.item(key, value);
    }

    put_item_request = put_item_request.item("pk", AttributeValue::S(item.pk()));

    if let Some(sk) = item.sk() {
        put_item_request = put_item_request.item("sk", AttributeValue::S(sk));
    }

    println!("putting item");
    let x = put_item_request.send().await;
    x.and_then(|_| Ok(()))
}

pub(crate) async fn get_room_by_id(client: &Client, id: &str) -> Option<Room> {
    let query_output = client
        .query()
        .table_name(MAIN_TABLE_NAME)
        .key_condition_expression("pk = :id")
        .expression_attribute_values(
            ":id",
            AttributeValue::S(Room::get_partition_key_from_id(id)),
        )
        .send()
        .await
        .ok();

    let Some(output) = query_output else {
        return None;
    };

    let Some(items) = output.items() else {
        return None;
    };

    let Some(room) = items.first() else {
        return None;
    };
    let room = room.clone();

    from_item(room).ok()
}
