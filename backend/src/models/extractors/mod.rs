use std::{future::Future, pin::Pin};

use super::User;
use async_trait::async_trait;
use axum::{extract::FromRequestParts, http::HeaderValue};
use rusty_ulid::Ulid;

pub(crate) struct UserExtractor(pub(crate) User);

impl UserExtractor {
    fn create_user_in_dynamo() -> Self {
        // TODO: create the user in dynamodb.
        Self(User::new(rusty_ulid::generate_ulid_string()))
    }
}

#[async_trait]
impl<S> FromRequestParts<S> for UserExtractor {
    type Rejection = ();

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        state: &S,
    ) -> Result<Self, Self::Rejection> {
        let user_header = match parts.headers.get("user") {
            // TODO: create item in the dynamodb as well.
            None => return Ok(Self::create_user_in_dynamo()),
            Some(user_header) => user_header,
        };

        let user = match user_header.to_str() {
            Ok(user_string) => User::from_str(user_string),
            Err(_) => {
                // TODO: create user in dynamodb.
                User::new(rusty_ulid::generate_ulid_string())
            }
        };

        Ok(Self(user))
    }
}
