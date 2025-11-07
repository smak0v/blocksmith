use actix_web::ResponseError;
use anyhow::anyhow;

use std::fmt::Display;

#[derive(Debug)]
pub struct ApiError(anyhow::Error);

impl Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl ResponseError for ApiError {}

impl From<anyhow::Error> for ApiError {
    fn from(error: anyhow::Error) -> Self {
        ApiError(error)
    }
}

impl From<secp256k1::Error> for ApiError {
    fn from(error: secp256k1::Error) -> Self {
        ApiError(anyhow!(error))
    }
}
