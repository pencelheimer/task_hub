use axum::http::StatusCode;
use loco_rs::{controller::ErrorDetail, errors::Error, Result};

pub fn unauthorized<T: Into<String>, U>(msg: T) -> Result<U> {
    Err(Error::Unauthorized(msg.into()))
}

pub fn conflict<T: Into<String>, U>(msg: T) -> Result<U> {
    Err(Error::CustomError(
        StatusCode::CONFLICT,
        ErrorDetail {
            error: Some("Conflict".to_string()),
            description: Some(msg.into()),
            errors: None,
        },
    ))
}

pub fn notfound<T: Into<String>, U>(msg: T) -> Result<U> {
    Err(Error::CustomError(
        StatusCode::NOT_FOUND,
        ErrorDetail {
            error: Some("Not Found".to_string()),
            description: Some(msg.into()),
            errors: None,
        },
    ))
}

pub fn internal<U>() -> Result<U> {
    Err(Error::InternalServerError)
}
