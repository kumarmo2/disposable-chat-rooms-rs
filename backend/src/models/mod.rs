#![allow(dead_code)]

pub(crate) mod extractors;

use std::{fmt::format, str::FromStr, time::Instant};

use aws_sdk_dynamodb::model::AttributeValue;
use serde::Deserialize;

use crate::dao::{BoxedAttributes, DynamoItem};

// use crate::dao::DynamoItem;

#[derive(Debug, Clone)]
pub(crate) struct User {
    pub(crate) id: String,
}

impl DynamoItem for User {
    fn attributes(&self) -> BoxedAttributes {
        Box::new(vec![("id", AttributeValue::S(self.id.to_string()))].into_iter())
    }

    fn pk(&self) -> String {
        format!("user|{}", self.id)
    }

    fn sk(&self) -> Option<String> {
        Some(format!("user|{}", self.id))
    }
}

impl User {
    pub(crate) fn new(id: String) -> Self {
        Self { id }
    }

    pub(crate) fn from_str(id: &str) -> Self {
        Self {
            id: String::from_str(id).unwrap(),
        }
    }

    pub(crate) fn get_partition_key(&self) -> String {
        format!("user|{}", self.id)
    }
    pub(crate) fn get_sort_key(&self) -> String {
        format!("user|{}", self.id)
    }
}

struct Member {}

#[derive(Debug, Deserialize)]
pub(crate) struct Room {
    pub(crate) id: String,
    display_name: String,
    created_by: String,
    // created_on: String,
}

impl Room {
    pub(crate) fn from_fields<T>(
        id: String,
        name: String,
        created_by: String,
        // created_on: Instant,
    ) -> Self
    where
        T: Into<String>,
    {
        Self {
            id,
            display_name: name,
            created_by,
            // TODO: handle for created_on
            // created_on: created_on.to_r,
        }
    }

    pub(crate) fn get_partition_key(&self) -> String {
        Self::get_partition_key_from_id(&self.id)
    }

    pub(crate) fn get_sort_key(&self) -> String {
        format!("room|{}", self.id)
    }

    pub(crate) fn name(&self) -> &str {
        self.display_name.as_str()
    }

    pub(crate) fn id(&self) -> &str {
        self.id.as_str()
    }

    pub(crate) fn get_partition_key_from_id(id: &str) -> String {
        format!("room|{}", id)
    }
}

impl DynamoItem for Room {
    fn attributes(&self) -> BoxedAttributes {
        Box::new(
            vec![
                ("id", AttributeValue::S(self.id.to_string())),
                (
                    "display_name",
                    AttributeValue::S(self.display_name.to_string()),
                ),
                ("created_by", AttributeValue::S(self.created_by.to_string())),
            ]
            .into_iter(),
        )
    }

    fn pk(&self) -> String {
        self.get_partition_key()
    }

    fn sk(&self) -> Option<String> {
        Some(self.get_sort_key())
    }
}
