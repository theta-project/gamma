use std::fmt::Write;

use actix_web::{http::StatusCode, web::BytesMut, HttpResponseBuilder};
use redis::RedisError;
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
    Internal(#[from] InternalError),
    #[error("external/client error: {}", .0)]
    External(#[from] ExternalError),
}

/// An error encountered by the server that should not have details shared with the client
/// eg the database not responding
#[derive(Debug, Error)]
pub enum InternalError {
    #[error("redis returned: {}", .0)]
    RedisError(#[from] RedisError),
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

impl RequestError {
    /// Log the given error. This can be used when the error is recoverable to make sure it's still acknowledged.
    pub fn report(&self) {
        error!(
            msg = "request ended in error",
            msg = self.to_string(),
            internal = matches!(self, RequestError::Internal(_))
        );
    }
}

impl actix_web::ResponseError for RequestError {
    fn status_code(&self) -> StatusCode {
        match self {
            RequestError::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
            RequestError::External(_) => StatusCode::BAD_REQUEST,
        }
    }

    fn error_response(&self) -> actix_web::HttpResponse<actix_web::body::BoxBody> {
        self.report(); // sus

        let mut res = HttpResponseBuilder::new(self.status_code());
        let mut buf = BytesMut::new();
        match self {
            RequestError::Internal(_) => buf.write_str("internal server error").unwrap(),
            RequestError::External(e) => buf.write_str(&format!("{}", e)).unwrap(),
        };

        res.content_type("text/plain").body(buf)
    }
}
