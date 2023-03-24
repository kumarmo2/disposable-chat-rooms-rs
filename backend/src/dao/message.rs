use aws_sdk_dynamodb::{error::QueryError, model::AttributeValue, types::SdkError, Client};
use serde_dynamo::aws_sdk_dynamodb_0_24::from_items;

use crate::models::message::Messsage;

use super::MAIN_TABLE_NAME;

pub(crate) async fn get_messages(
    room_id: &str,
    client: &Client,
) -> Result<Vec<Messsage>, SdkError<QueryError>> {
    let pk = AttributeValue::S(Messsage::get_partition_key_from_room_id(room_id));
    let start_value = AttributeValue::S("message".to_string());
    let end_value = AttributeValue::S("n".to_string());

    println!("pk: {:?}", pk);
    let query_output = client
        .query()
        .table_name(MAIN_TABLE_NAME)
        .key_condition_expression("pk = :pk and sk between :start and :end")
        .expression_attribute_values(":start", start_value)
        .expression_attribute_values(":end", end_value)
        .expression_attribute_values(":pk", pk)
        // scan_index_forward = false, for reverse order
        .scan_index_forward(false)
        .send()
        .await?;

    if let Some(items) = query_output.items().map(|items| items.to_vec()) {
        println!("found some messages");
        let messages: Vec<Messsage> = from_items(items).unwrap();
        return Ok(messages);
    }

    Ok(vec![])
}
