#![warn(dead_code)]

use std::pin::Pin;
use std::{future::Future, time::Duration};

use crate::models::User;
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
    <T as Service<Request<B>>>::Response: IntoResponse,
{
    type Response = (Option<CookieJar>, T::Response);
    // type Response = ;

    type Error = T::Error;

    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        println!("inside poll_ready");
        return self.inner_service.poll_ready(cx);
    }

    fn call(&mut self, mut req: Request<B>) -> Self::Future {
        println!("inside call");
        let mut jar = CookieJar::from_headers(req.headers());
        if let Some(user_cookie) = jar.get("user") {
            println!("found cookie, inserting the user extension");
            let user = User::from_str(user_cookie.value());
            req.extensions_mut().insert(user);
            let future = self.inner_service.call(req);
            let fut = async {
                if let Ok(res) = future.await {
                    let result: Result<Self::Response, Self::Error> = Ok((None, res));
                    return result;
                    // return Box::pin(result);
                }
                todo!();
            };
            return Box::pin(fut);

            // return Box::pin(future);
        }
        todo!();
        let user_id = rusty_ulid::generate_ulid_string();
        std::thread::sleep(Duration::from_secs(2));

        println!("didn't find cookie, inserting into extension");
        req.extensions_mut()
            .insert(User::from_str(user_id.as_str()));

        // let fut = self.inner_service.call(req);
        // Box::pin(fut)
    }
}
