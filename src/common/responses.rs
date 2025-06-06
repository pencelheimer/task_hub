use axum::http::StatusCode;
use loco_rs::{controller::ErrorDetail, errors::Error, Result};

pub fn unauthorized<T: Into<String>, U>(msg: T) -> Result<U> {
    Err(Error::CustomError(
        StatusCode::UNAUTHORIZED,
        ErrorDetail {
            error: Some("Unauthorised".to_string()),
            description: Some(msg.into()),
            errors: None,
        },
    ))
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

pub fn forbidden<T: Into<String>, U>(msg: T) -> Result<U> {
    Err(Error::CustomError(
        StatusCode::FORBIDDEN,
        ErrorDetail {
            error: Some("Forbidden".to_string()),
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

pub fn bad_request<T: Into<String>, U>(msg: T) -> Result<U> {
    Err(Error::BadRequest(msg.into()))
}
