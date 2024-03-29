use std::{collections::HashMap, ops::Deref, str::FromStr};
pub(crate) mod message;
pub(crate) mod ping;
pub(crate) mod room;

use aws_sdk_dynamodb::{
    error::{PutItemError, QueryError},
    model::AttributeValue,
    types::SdkError,
    Client,
};
use serde::Deserialize;
use serde_dynamo::from_item;

use crate::dtos::DaoError;

const MAIN_TABLE_NAME: &'static str = "main";

pub(crate) type BoxedAttributes = Box<dyn Iterator<Item = (&'static str, AttributeValue)>>;

pub(crate) trait DynamoItem {
    // NOTE: attributes shouldn't send the "pk" and "sk". For them, we have another trait methods.
    fn attributes(&self) -> BoxedAttributes;
    fn pk(&self) -> String;
    fn sk(&self) -> Option<String>;
}

pub(crate) async fn get_item_by_primary_key<'a, T, S1, S2>(
    client: &Client,
    partition_key: S1,
    sort_key: Option<S2>,
) -> Result<Option<T>, DaoError>
where
    T: Deserialize<'a>,
    S1: AsRef<str>,
    S2: AsRef<str>,
{
    let mut key_condition_expression = String::from_str("pk = :pk").unwrap();
    if let Some(_) = sort_key.as_ref() {
        key_condition_expression.push_str(" and sk = :sk");
    };

    let mut query = client
        .query()
        .table_name(MAIN_TABLE_NAME)
        .key_condition_expression(key_condition_expression)
        .expression_attribute_values(":pk", AttributeValue::S(partition_key.as_ref().to_string()));

    if let Some(sk) = sort_key {
        query = query.expression_attribute_values(":sk", AttributeValue::S(sk.as_ref().to_string()))
    }

    let query_output = query.send().await;

    if let Err(e) = query_output {
        println!("error while putting item, err: {:?}", e);
        return Err(DaoError::QueryError(e));
    };
    let output = query_output.unwrap();

    let Some(items) = output.items() else{
        return Ok(None);

    };

    let Some(item) = items.first() else {
        return Ok(None);
    };

    from_item(item.clone())
        .or_else(|e| {
            let error = format!("error while deserializing item, e: {:?}", e);
            println!("{}", error);
            Err(DaoError::Internal(error))
        })
        .and_then(|item| Ok(item))
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

    // println!("putting item, {:?}", put_item_request);
    let x = put_item_request.send().await;
    x.and_then(|_| Ok(())).or_else(|x| {
        println!("error, x: {:?}", x);
        Err(x)
    })
}
