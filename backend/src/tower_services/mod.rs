#![warn(dead_code)]

use std::pin::Pin;
use std::{future::Future, time::Duration};

use crate::models::User;
use crate::AppState;
use axum::body::Body;
use axum::http::request::Request;
use axum::response::IntoResponse;
use axum_extra::extract::cookie::Cookie;
use axum_extra::extract::CookieJar;
use tower::{Layer, Service};

#[derive(Clone)]
pub(crate) struct UserService<T> {
    inner_service: T,
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
        };
    }
}

impl<T> Service<Request<Body>> for UserService<T>
where
    T: Service<Request<Body>> + Send,
    <T as Service<Request<Body>>>::Future: Send + 'static,
    <T as Service<Request<Body>>>::Response: IntoResponse,
{
    type Response = (Option<CookieJar>, T::Response);

    type Error = T::Error;

    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        println!("inside poll_ready");
        return self.inner_service.poll_ready(cx);
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
            let fut = async move { fut.await.and_then(|res| Ok((None, res))) };
            return Box::pin(fut);
        }

        println!("didn't find cookie");
        let id = rusty_ulid::generate_ulid_string();
        let user = User::new(id.clone());
        req.extensions_mut().insert(user);
        let mut jar = CookieJar::from_headers(req.headers());
        let future = self.inner_service.call(req);
        let fut = async move {
            match future.await {
                Err(e) => Err(e),
                Ok(r) => {
                    jar = jar.add(Cookie::new("user", id.clone()));
                    Ok((Some(jar), r))
                }
            }
        };
        Box::pin(fut)
    }
}
