#![warn(dead_code)]

use std::pin::Pin;
use std::task::Poll;
use std::{future::Future, time::Duration};

use crate::dao::put_user;
use crate::models::User;
use crate::AppState;
use aws_sdk_dynamodb::Client;
use axum::body::Body;
use axum::http::request::Request;
use axum::response::IntoResponse;
use axum_extra::extract::cookie::Cookie;
use axum_extra::extract::CookieJar;
use hyper::StatusCode;
use tower::{Layer, Service};

#[derive(Clone)]
pub(crate) struct UserService<T> {
    inner_service: T,
    dynamodb: AppState,
}

#[derive(Clone)]
pub(crate) struct UserLayer(pub(crate) AppState);

impl<T> Layer<T> for UserLayer
where
    T: Service<Request<Body>>,
{
    type Service = UserService<T>;

    fn layer(&self, inner: T) -> Self::Service {
        return UserService {
            inner_service: inner,
            dynamodb: self.0.clone(),
        };
    }
}

pub(crate) enum Error<E> {
    ServiceError(E),
    DynamoDbPutItemError(StatusCode),
}

impl<T> Service<Request<Body>> for UserService<T>
where
    T: Service<Request<Body>> + Send,
    <T as Service<Request<Body>>>::Future: Send + 'static,
    <T as Service<Request<Body>>>::Response: IntoResponse,
{
    type Response = (Option<CookieJar>, T::Response);

    // type Error = T::Error;
    type Error = Error<T::Error>;

    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        println!("inside poll_ready");
        let result = match self.inner_service.poll_ready(cx) {
            Poll::Pending => return Poll::Pending,
            Poll::Ready(res) => res
                .and_then(|_| Ok(()))
                .or_else(|err| Err(Error::ServiceError(err))),
        };
        Poll::Ready(result)
    }

    fn call(&mut self, mut req: Request<Body>) -> Self::Future {
        // TODO:
        // 1. if user cookie is already present, then just create User from the existing cookie instead of generatin new one.
        // 2. persis the user into dynamodb.
        println!("inside call");

        let jar = CookieJar::from_headers(req.headers());
        if let Some(id) = jar.get("user").and_then(|x| Some(x.value())) {
            println!("found user cookie, id: {}", id);
            let user = User::from_str(id);
            req.extensions_mut().insert(user);
            let fut = self.inner_service.call(req);
            let fut = async move {
                fut.await
                    .and_then(|res| Ok((None, res)))
                    .or_else(|err| Err(Error::ServiceError(err)))
            };
            return Box::pin(fut);
        }

        println!("didn't find cookie");
        let id = rusty_ulid::generate_ulid_string();
        let user = User::new(id.clone());

        let fut = async move {
            match put_user(&self.dynamodb.lock().await.dynamodb, &user).await {
                Err(e) => {
                    return Err(Error::<T::Error>::DynamoDbPutItemError(
                        StatusCode::INTERNAL_SERVER_ERROR,
                    ))
                }
                _ => (),
            };
            let mut jar = CookieJar::from_headers(req.headers());

            req.extensions_mut().insert(user);
            let x: Result<
                (Option<CookieJar>, <T as Service<Request<Body>>>::Response),
                Error<<T as Service<Request<Body>>>::Error>,
            > = self
                .inner_service
                .call(req)
                .await
                .and_then(|res| {
                    let jar = jar.add(Cookie::new("user", id));
                    Ok((Some(jar), res))
                })
                .or_else(|e| Err(Error::ServiceError(e)));
            return x;
        };

        Box::pin(fut)
    }
}
