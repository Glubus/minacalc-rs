use std::fmt;

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;

#[derive(Debug)]
pub(crate) struct ApiError {
    status: StatusCode,
    message: String,
}

impl ApiError {
    pub(crate) fn bad_request(message: impl Into<String>) -> Self {
        Self::new(StatusCode::BAD_REQUEST, message)
    }

    pub(crate) fn payload_too_large(message: impl Into<String>) -> Self {
        Self::new(StatusCode::PAYLOAD_TOO_LARGE, message)
    }

    pub(crate) fn bad_gateway(message: impl Into<String>) -> Self {
        Self::new(StatusCode::BAD_GATEWAY, message)
    }

    pub(crate) fn internal(message: impl Into<String>) -> Self {
        Self::new(StatusCode::INTERNAL_SERVER_ERROR, message)
    }

    fn new(status: StatusCode, message: impl Into<String>) -> Self {
        Self {
            status,
            message: message.into(),
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let body = Json(ErrorResponse {
            error: self.message,
        });
        (self.status, body).into_response()
    }
}

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
}

/// Errors raised while converting ROX notes to MinaCalc rows.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConversionError {
    EmptyChart,
    NoPlayableNotes,
    UnsupportedKeyCount(u8),
    InvalidColumn { column: u8, key_count: u8 },
}

impl fmt::Display for ConversionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyChart => formatter.write_str("chart has no notes"),
            Self::NoPlayableNotes => formatter.write_str("chart has no playable notes"),
            Self::UnsupportedKeyCount(key_count) => write!(
                formatter,
                "unsupported key count {key_count}; MinaCalc supports 4K, 6K, and 7K"
            ),
            Self::InvalidColumn { column, key_count } => write!(
                formatter,
                "note column {column} is outside the chart's {key_count} columns"
            ),
        }
    }
}

impl std::error::Error for ConversionError {}
