use std::vec;

use aws_sdk_dynamodb::model::AttributeValue::S;
use serde::{Deserialize, Serialize};

use crate::dao::DynamoItem;

#[derive(Serialize, Deserialize)]
pub(crate) struct Member {
    pub(crate) display_name: String,
    room_id: String,
    user_id: String,
}

impl Member {
    pub(crate) fn from_fields(display_name: String, room_id: String, user_id: String) -> Self {
        Self {
            display_name,
            room_id,
            user_id,
        }
    }

    pub(crate) fn get_partition_key_from_room_id(id: &str) -> String {
        format!("room|{}", id)
    }

    pub(crate) fn get_sort_key_from_user_id<T>(id: &T) -> String
    where
        T: AsRef<str>,
    {
        format!("user|{}", id.as_ref())
    }
}

impl DynamoItem for Member {
    fn attributes(&self) -> crate::dao::BoxedAttributes {
        Box::new(
            vec![
                ("display_name", S(self.display_name.to_string())),
                ("room_id", S(self.room_id.to_string())),
                ("user_id", S(self.user_id.to_string())),
            ]
            .into_iter(),
        )
    }

    fn pk(&self) -> String {
        Self::get_partition_key_from_room_id(&self.room_id)
    }

    fn sk(&self) -> Option<String> {
        Some(Self::get_sort_key_from_user_id(&self.user_id))
    }
}
