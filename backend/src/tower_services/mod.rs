#![warn(dead_code)]

use std::pin::Pin;
use std::{future::Future, time::Duration};

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
pub(crate) struct UserLayer;

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

impl<T, B> Service<Request<B>> for UserService<T>
where
    T: Service<Request<B>>,
    <T as Service<Request<B>>>::Future: Send + 'static,
{
    type Response = T::Response;

    type Error = T::Error;

    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        println!("inside poll_ready");
        return self.inner_service.poll_ready(cx);
    }

    fn call(&mut self, req: Request<B>) -> Self::Future {
        println!("inside call");
        let mut jar = CookieJar::from_headers(req.headers());
        if let Some(_) = jar.get("user") {
            let future = self.inner_service.call(req);
            return Box::pin(future);
        }
        let user_id = rusty_ulid::generate_ulid_string();
        let user_cookie = Cookie::new("user", user_id);
        let cookie = user_cookie.to_string();
        println!("cookie: {}", cookie);
        std::thread::sleep(Duration::from_secs(2));
        // Box::pin(tokio::time::sleep(Duration::from_secs(2)));
        // jar = jar.add(user_cookie);
        // jar.into
        // req.headers_mut().

        panic!()
    }
}
