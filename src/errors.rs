//! Error handling logic
//! This uses some tricks to basically make it so that as soon as an error is constructed, it is logged
//! Meaning the log entry has all the context from the above spans, so we already know the context, etc.
//! This is sort of inspired by `anyhow`'s `.context()`, but utilising `tracing` to make it all semi-automatic.
//!
//! Because there are no such thing as private enum constructors in rust, we use unit structs to make it so that
//! a `RequestError` can only be constructed through the two `From` implementations.
//! They serve no purpose other than this.
//!
//! If you want to log an error, but keep going, you'll still need to use the result, so something like:
//! ```
//! let _ = RequestError::from(e as InternalError)
//! ```
//! Otherwise, you can mostly just use `Result` and sprinkle `?` in everywhere

use actix_web::{http::StatusCode, web::BytesMut, HttpResponseBuilder};
use redis::RedisError;
use std::fmt::Write;
use thiserror::Error;
use tracing::error;

/// Convenience type for using RequestError
pub type Result<O, E = RequestError> = std::result::Result<O, E>;

/// An error that can be encountered during a request
/// This is either an internal one, which will be hidden to the user
/// or an external one, which will be returned
#[derive(Debug, Error)]
pub enum RequestError {
    #[error("internal error: {}", .0)]
    Internal(__InternalError),
    #[error("external/client error: {}", .0)]
    External(__ExternalError),
}

#[derive(Debug, Error)]
#[error(transparent)]
pub struct __InternalError(InternalError);

#[derive(Debug, Error)]
#[error(transparent)]
pub struct __ExternalError(ExternalError);

/// An error encountered by the server that should not have details shared with the client
/// eg the database not responding
#[derive(Debug, Error)]
pub enum InternalError {
    #[error("redis returned: {}", .0)]
    Redis(#[from] RedisError),

    #[error("could not get from redis connection pool: {}", .0)]
    RedisPool(#[from] deadpool_redis::PoolError),

    #[error("could not get from sql connection pool: {}", .0)]
    SqlPool(#[from] sqlx::Error),
}

/// An error encountered by the server that it's ok to share details with the client about
/// eg a malformed packet
#[derive(Debug, Error)]
pub enum ExternalError {
    #[error("malformed packet: {}", .0)]
    MalformedPacket(&'static str),
    #[error("token is invalid")]
    InvalidToken,
}

impl actix_web::ResponseError for RequestError {
    fn status_code(&self) -> StatusCode {
        match self {
            RequestError::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
            RequestError::External(_) => StatusCode::BAD_REQUEST,
        }
    }

    fn error_response(&self) -> actix_web::HttpResponse<actix_web::body::BoxBody> {
        let mut res = HttpResponseBuilder::new(self.status_code());
        let mut buf = BytesMut::new();
        match self {
            RequestError::Internal(_) => buf.write_str("internal server error").unwrap(),
            RequestError::External(e) => buf.write_str(&format!("{}", e)).unwrap(),
        };

        res.content_type("text/plain").body(buf)
    }
}

impl From<InternalError> for RequestError {
    fn from(err: InternalError) -> Self {
        error!(err = err.to_string(), internal = true);
        RequestError::Internal(__InternalError(err))
    }
}

impl From<ExternalError> for RequestError {
    fn from(err: ExternalError) -> Self {
        error!(err = err.to_string(), internal = false);
        RequestError::External(__ExternalError(err))
    }
}
