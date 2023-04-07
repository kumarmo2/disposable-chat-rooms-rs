use std::{net::IpAddr, vec};

use aws_sdk_dynamodb::model::AttributeValue;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::dao::DynamoItem;

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct Ping {
    pub(crate) user_id: String,
    pub(crate) valid_till: DateTime<Utc>,
    pub(crate) server_ip: IpAddr,
}

impl DynamoItem for Ping {
    fn attributes(&self) -> crate::dao::BoxedAttributes {
        Box::new(
            vec![
                ("user_id", AttributeValue::S(self.user_id.to_string())),
                (
                    "server_ip",
                    AttributeValue::S(serde_json::to_string(&self.server_ip).unwrap()),
                ),
                (
                    "valid_till",
                    AttributeValue::S(serde_json::to_string(&self.valid_till).unwrap()),
                ),
            ]
            .into_iter(),
        )
    }

    fn pk(&self) -> String {
        format!("ping_user|{}", self.user_id)
    }

    fn sk(&self) -> Option<String> {
        None
    }
}
