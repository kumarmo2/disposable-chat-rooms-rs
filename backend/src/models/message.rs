use aws_sdk_dynamodb::model::AttributeValue;
// use aws_sdk_dynamodb::model::AttributeValueUpdate;
use serde::{Deserialize, Serialize};
// use serde_dynamo::AttributeValue;

use crate::dao::DynamoItem;

use super::Room;

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct Messsage {
    pub(crate) id: String,
    pub(crate) room_id: String,
    pub(crate) content: String,
    pub(crate) sent_by_user_id: String,
    pub(crate) sent_by_member_name: String,
}

impl Messsage {
    pub(crate) fn from_fields(
        id: String,
        room_id: String,
        content: String,
        sent_by_user_id: String,
        sent_by_member_name: String,
    ) -> Self {
        Self {
            id,
            room_id,
            content,
            sent_by_user_id,
            sent_by_member_name,
        }
    }

    pub(crate) fn get_sort_key_from_id(message_id: &str) -> String {
        format!("message|{}", message_id)
    }

    pub(crate) fn get_partition_key_from_room_id(id: &str) -> String {
        Room::get_partition_key_from_id(id)
    }
}

impl DynamoItem for Messsage {
    fn attributes(&self) -> crate::dao::BoxedAttributes {
        Box::new(
            vec![
                ("id", AttributeValue::S(self.id.to_string())),
                ("room_id", AttributeValue::S(self.room_id.to_string())),
                (
                    "sent_by_member_name",
                    AttributeValue::S(self.sent_by_member_name.to_string()),
                ),
                (
                    "sent_by_user_id",
                    AttributeValue::S(self.sent_by_user_id.to_string()),
                ),
                ("content", AttributeValue::S(self.content.to_string())),
            ]
            .into_iter(),
        )
    }

    fn pk(&self) -> String {
        Self::get_partition_key_from_room_id(&self.room_id)
    }

    fn sk(&self) -> Option<String> {
        Some(Self::get_sort_key_from_id(&self.id))
    }
}
