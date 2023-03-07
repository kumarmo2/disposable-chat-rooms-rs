#![warn(dead_code)]

use std::future::Future;
use std::pin::Pin;
use std::task::Poll;

use crate::dao::put_item;
use crate::models::User;
use crate::AppState;
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

impl<E> IntoResponse for Error<E>
where
    E: IntoResponse,
{
    fn into_response(self) -> axum::response::Response {
        match self {
            Error::ServiceError(err) => err.into_response(),
            Error::DynamoDbPutItemError(code) => code.into_response(),
        }
    }
}

impl<T> Service<Request<Body>> for UserService<T>
where
    T: Service<Request<Body>> + Send + Clone,
    <T as Service<Request<Body>>>::Future: Send + 'static,
    <T as Service<Request<Body>>>::Response: IntoResponse,
{
    type Response = (Option<CookieJar>, T::Response);
    type Error = Error<T::Error>;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        let result = match self.inner_service.poll_ready(cx) {
            Poll::Pending => return Poll::Pending,
            Poll::Ready(res) => res
                .and_then(|_| Ok(()))
                .or_else(|err| Err(Error::ServiceError(err))),
        };
        Poll::Ready(result)
    }

    fn call(&mut self, mut req: Request<Body>) -> Self::Future {
        let mut jar = CookieJar::from_headers(req.headers());
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

        let id = rusty_ulid::generate_ulid_string();
        let user = User::new(id.clone());
        req.extensions_mut().insert(user.clone());
        let fut = self.inner_service.call(req);

        let cloned_self = self.clone();
        let fut = async move {
            match put_item(&cloned_self.dynamodb.lock().await.dynamodb, &user).await {
                Err(_) => {
                    println!("put item dynamodb request failed");
                    return Err(Error::<T::Error>::DynamoDbPutItemError(
                        StatusCode::INTERNAL_SERVER_ERROR,
                    ));
                }
                _ => (),
            };

            let x = fut
                .await
                .and_then(|res| {
                    jar = jar.add(Cookie::new("user", id));
                    Ok((Some(jar), res))
                })
                .or_else(|e| Err(Error::ServiceError(e)));
            return x;
        };

        Box::pin(fut)
    }
}
