#![allow(dead_code)]

pub(crate) mod extractors;

use std::str::FromStr;

use rusty_ulid::generate_ulid_string;

#[derive(Debug, Clone)]
pub(crate) struct User {
    pub(crate) id: String,
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

#[derive(Debug)]
pub(crate) struct Room {
    id: String,
    name: String,
}

impl Room {
    pub(crate) fn new(name: &str) -> Self {
        Self {
            id: generate_ulid_string(),
            name: name.to_owned(),
        }
    }

    pub(crate) fn from_fields(id: String, name: String) -> Self {
        Self { id, name }
    }

    pub(crate) fn get_partition_key(&self) -> String {
        format!("room|{}", self.id)
    }

    pub(crate) fn get_sort_key(&self) -> String {
        format!("room|{}", self.id)
    }

    pub(crate) fn name(&self) -> &str {
        self.name.as_str()
    }

    pub(crate) fn id(&self) -> &str {
        self.id.as_str()
    }
}
