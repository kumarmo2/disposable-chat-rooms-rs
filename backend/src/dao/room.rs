use aws_sdk_dynamodb::{
    error::QueryError, model::AttributeValue, output::query_output, types::SdkError, Client,
};
use serde_dynamo::aws_sdk_dynamodb_0_24::{from_item, from_items};

use crate::models::{member::Member, Room};

use super::MAIN_TABLE_NAME;

pub(crate) async fn get_room_by_id(client: &Client, id: &str) -> Option<Room> {
    let query_output = client
        .query()
        .table_name(MAIN_TABLE_NAME)
        .key_condition_expression("pk = :id and sk = :id")
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

pub(crate) async fn get_rooom_members(
    client: &Client,
    id: &str,
) -> Result<Vec<Member>, SdkError<QueryError>> {
    let output = client
        .query()
        .table_name(MAIN_TABLE_NAME)
        .key_condition_expression("pk = :id and sk between :start and :end")
        .expression_attribute_values(
            ":id",
            AttributeValue::S(Room::get_partition_key_from_id(id)),
        )
        .expression_attribute_values(":start", AttributeValue::S(format!("user",)))
        .expression_attribute_values(":end", AttributeValue::S(format!("v")))
        .send()
        .await?;

    if let Some(items) = output.items().map(|items| items.to_vec()) {
        // TODO: get rid of this unwrap.
        let members: Vec<Member> = from_items(items).unwrap();
        return Ok(members);
    };
    Ok(vec![])
}
