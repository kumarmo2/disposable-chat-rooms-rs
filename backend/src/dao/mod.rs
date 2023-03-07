use aws_sdk_dynamodb::{error::PutItemError, model::AttributeValue, types::SdkError, Client};

pub(crate) type BoxedAttributes = Box<dyn Iterator<Item = (&'static str, AttributeValue)>>;

pub(crate) trait DynamoItem {
    // NOTE: attributes shouldn't send the "pk" and "sk". For them, we have another trait methods.
    fn attributes(&self) -> BoxedAttributes;
    fn pk(&self) -> String;
    fn sk(&self) -> String;
}

pub(crate) async fn put_item<T>(client: &Client, user: &T) -> Result<(), SdkError<PutItemError>>
where
    T: DynamoItem,
{
    let mut put_item_request = client.put_item().table_name("main");

    for (key, value) in user.attributes() {
        put_item_request = put_item_request.item(key, value);
    }

    put_item_request = put_item_request
        .item("pk", AttributeValue::S(user.pk()))
        .item("sk", AttributeValue::S(user.sk()));

    println!("putting item");
    let x = put_item_request.send().await;
    x.and_then(|_| Ok(()))
}