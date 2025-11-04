//! Business logic services
//!
//! This module contains business logic separated from HTTP concerns.
//! Services are pure functions or stateless operations that can be
//! easily tested and reused.

use actix_web::HttpResponse;

pub mod auth;
pub mod certificate;
pub mod firmware;
pub mod network;

/// Trait for converting service results into HTTP responses
pub trait ServiceResultResponse {
    fn into_response(self) -> HttpResponse;
}

impl ServiceResultResponse for () {
    fn into_response(self) -> HttpResponse {
        HttpResponse::Ok().finish()
    }
}

impl ServiceResultResponse for String {
    fn into_response(self) -> HttpResponse {
        HttpResponse::Ok().body(self)
    }
}
