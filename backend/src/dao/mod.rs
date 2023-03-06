use aws_sdk_dynamodb::{error::PutItemError, model::AttributeValue, types::SdkError, Client};

use crate::models::User;

pub(crate) async fn put_user(client: &Client, user: &User) -> Result<(), SdkError<PutItemError>> {
    let attrs = [
        ("pk", AttributeValue::S(user.get_partition_key())),
        ("sk", AttributeValue::S(user.get_sort_key())),
        ("id", AttributeValue::S(user.id.to_string())),
    ];

    let mut put_item_request = client.put_item().table_name("main");

    for (key, value) in attrs {
        put_item_request = put_item_request.item(key, value);
    }

    let x = put_item_request.send().await;
    x.and_then(|_| Ok(()))
}
