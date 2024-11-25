use actix_web::{
    body::EitherBody,
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error, FromRequest, HttpMessage, HttpResponse,
};
use actix_web_httpauth::extractors::bearer::BearerAuth;
use anyhow::{Context, Result};
use jwt_simple::prelude::*;
use log::error;
use std::{
    future::{ready, Future, Ready},
    pin::Pin,
    rc::Rc,
};

pub const TOKEN_EXPIRE_HOURES: u64 = 2;

pub struct Auth;

impl<S, B> Transform<S, ServiceRequest> for Auth
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type InitError = ();
    type Transform = AuthMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(AuthMiddleware {
            service: Rc::new(service),
        }))
    }
}

pub struct AuthMiddleware<S> {
    service: Rc<S>,
}

type LocalBoxFuture<T> = Pin<Box<dyn Future<Output = T> + 'static>>;

impl<S, B> Service<ServiceRequest> for AuthMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type Future = LocalBoxFuture<Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, mut req: ServiceRequest) -> Self::Future {
        let service = Rc::clone(&self.service);

        Box::pin(async move {
            if str::starts_with(req.path(), "/action") {
                let mut payload = req.take_payload().take();

                let auth = match BearerAuth::from_request(req.request(), &mut payload).await {
                    Ok(b) => b,
                    Err(_) => {
                        error!("No auth header");
                        return Ok(unauthorized_error(req).map_into_right_body());
                    }
                };

                match verify_token(auth) {
                    Ok(_) => (),
                    Err(e) => {
                        error!("User not authorized {}", e);

                        return Ok(unauthorized_error(req).map_into_right_body());
                    }
                }
            }

            let res = service.call(req).await?;
            Ok(res.map_into_left_body())
        })
    }
}

pub fn verify_token(auth: BearerAuth) -> Result<bool> {
    let key = std::env::var("CENTRIFUGO_TOKEN_HMAC_SECRET_KEY").context("missing jwt secret")?;
    let key = HS256Key::from_bytes(key.as_bytes());
    let options = VerificationOptions {
        accept_future: true,
        time_tolerance: Some(Duration::from_mins(15)),
        max_validity: Some(Duration::from_hours(TOKEN_EXPIRE_HOURES)),
        required_subject: Some("omnect-ui".to_string()),
        ..Default::default()
    };

    Ok(key
        .verify_token::<NoCustomClaims>(auth.token(), Some(options))
        .is_ok())
}

fn unauthorized_error(req: ServiceRequest) -> ServiceResponse {
    let http_res = HttpResponse::Unauthorized().finish();
    let (http_req, _) = req.into_parts();
    ServiceResponse::new(http_req, http_res)
}
