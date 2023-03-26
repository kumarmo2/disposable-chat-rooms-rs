use std::pin::Pin;
use std::sync::Arc;

use axum::http::Request;
// use axum::response::IntoResponse;
use axum_extra::extract::CookieJar;
use hyper::{Body, StatusCode};
use tower::{Layer, Service};

use crate::dao;
use crate::dtos::AppState;
use crate::models::User;

#[derive(Clone)]
pub(crate) struct EventsAuthService<T>
where
    T: Clone,
{
    inner_service: T,
    app_state: AppState,
}

#[derive(Clone)]
pub(crate) struct EventsAuthLayer {
    pub(crate) app_state: AppState,
}

impl<S> Layer<S> for EventsAuthLayer
where
    S: Clone,
{
    type Service = EventsAuthService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        EventsAuthService {
            inner_service: inner,
            app_state: Arc::clone(&self.app_state),
        }
    }
}

impl<T> Service<Request<Body>> for EventsAuthService<T>
where
    T: Service<Request<Body>> + Clone + 'static + Send,
    <T as Service<Request<Body>>>::Future: Send + 'static,
{
    type Response = Result<T::Response, StatusCode>;

    type Error = T::Error;

    type Future =
        Pin<Box<dyn std::future::Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.inner_service.poll_ready(cx)
    }

    fn call(&mut self, req: Request<Body>) -> Self::Future {
        /*
         * - check if "user" cookie exits
         * - if no, return unauthorized.
         * - get the userid from cookie, and check if in dynamo that exists.
         * - if no, return unauthorized.
         * - if any error in dynamo, return internal server error.
         *  - call inner service and return its response.
         *
         *
         * */

        let cookies = CookieJar::from_headers(req.headers());
        let Some(user_cookie) = cookies.get("user") else {
            let res = async {
                Ok(Err(StatusCode::UNAUTHORIZED))
            };
            return Box::pin(res);
        };
        let mut cloned_self = self.clone();
        let user_id = user_cookie.value().to_string();
        let res = async move {
            let partition_key = User::get_parition_key_from_user_id(&user_id);
            let user = dao::get_item_by_primary_key::<User>(
                &cloned_self.app_state.dynamodb,
                &partition_key,
                Some(&partition_key),
            )
            .await
            .map_err(|err| {
                println!("error in the auth service. err: {:?}", err);
                ()
            })
            .ok();
            if let None = user {
                println!("user could not be found");
                return Ok(Err(StatusCode::UNAUTHORIZED));
            };
            // TODO: refactor this match expression.
            match cloned_self.inner_service.call(req).await {
                Ok(res) => Ok(Ok(res)),
                Err(e) => {
                    // println!("internal error, e: {:?}", e);
                    Err(e)
                }
            }
        };
        Box::pin(res)
    }
}
