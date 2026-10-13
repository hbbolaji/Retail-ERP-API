use std::result;

use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use chromiumoxide::error::CdpError;
use scraper::{Selector, error::SelectorErrorKind};
use serde::Serialize;
use thiserror::Error;

use crate::error;

pub type Result<T> = result::Result<T, AppError>;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("not found")]
    NotFound(String),

    #[error("bad request")]
    BadRequest(String),

    #[error("something went wrong on our end")]
    InternalError(String),

    #[error("parse error")]
    ChromeError(#[from] CdpError),
    // #[error("scrapper error")]
    // ScrapperError(#[from] scraper::error::E),
}

#[derive(Serialize)]
struct ErrorBody {
    error: String,
    code: u16,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            Self::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
            Self::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg),
            Self::InternalError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
            Self::ChromeError(err) => (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()),
            // Self::ScrapperError(err) => (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()),
        };

        let body = ErrorBody {
            error: message,
            code: status.as_u16(),
        };

        (status, Json(body)).into_response()
    }
}
