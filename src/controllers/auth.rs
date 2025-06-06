use crate::{
    common::{responses, settings::Settings},
    mailers::auth::AuthMailer,
    models::{
        _entities::users,
        users::{LoginParams, RegisterParams},
    },
    views::auth::{CurrentResponse, LoginResponse},
};

use axum::debug_handler;
use loco_openapi::prelude::*;
use loco_rs::prelude::*;
use tower_cookies::{cookie::SameSite, Cookie, Cookies};

use regex::Regex;
use serde::{Deserialize, Serialize};
use std::sync::OnceLock;
use time;

pub static EMAIL_DOMAIN_RE: OnceLock<Regex> = OnceLock::new();

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct ForgotParams {
    pub email: String,
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct ResetParams {
    pub token: String,
    pub password: String,
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct MagicLinkParams {
    pub email: String,
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct ResendVerificationParams {
    pub email: String,
}

/// Register user
///
/// Register function creates a new user with the given parameters and sends a
/// welcome email to the user
#[utoipa::path(
    post,
    path = "/api/auth/register",
    tag = "auth",
    responses(
        (status = 200, description = "User is registered"),
        (status = 409, description = "User with given email already exist"),
        (status = 500, description = "Internal server error")
    ),
    request_body = RegisterParams
)]
#[debug_handler]
async fn register(
    State(ctx): State<AppContext>,
    Json(params): Json<RegisterParams>,
) -> Result<impl IntoResponse> {
    let res = users::Model::create_with_password(&ctx.db, &params).await;

    let user = match res {
        Ok(user) => user,
        Err(err) => {
            let msg = err.to_string();

            tracing::warn!(
                message = &msg,
                user_email = &params.email,
                "could not register user",
            );

            match err {
                ModelError::EntityAlreadyExists => return responses::conflict(msg),
                ModelError::EntityNotFound => return responses::notfound(msg),
                _ => return responses::internal(),
            };
        }
    };

    let user = user
        .into_active_model()
        .set_email_verification_token(&ctx.db)
        .await?;

    AuthMailer::send_welcome(&ctx, &user).await?;

    format::empty()
}

/// Resend verification email
///
/// Resends welcome email to the user
#[utoipa::path(
    post,
    path = "/api/auth/resend-verification-mail",
    tag = "auth",
    responses(
        (status = 200, description = "Verification email is sent"),
        (status = 404, description = "User with given email is not registered"),
        (status = 409, description = "User already verified"),
        (status = 500, description = "Internal server error")
    ),
    request_body = RegisterParams
)]
#[debug_handler]
async fn resend_verification_email(
    State(ctx): State<AppContext>,
    Json(params): Json<ResendVerificationParams>,
) -> Result<Response> {
    let Ok(user) = users::Model::find_by_email(&ctx.db, &params.email).await else {
        return responses::notfound("User not found for resend verification");
    };

    if user.email_verified_at.is_some() {
        return responses::conflict("User already verified");
    }

    let user = user
        .into_active_model()
        .set_email_verification_token(&ctx.db)
        .await?;

    AuthMailer::send_welcome(&ctx, &user).await?;

    format::empty()
}

/// Verify user
///
/// Verify register user.
#[utoipa::path(
    post,
    path = "/api/auth/verify",
    tag = "auth",
    responses(
        (status = 303, description = "User email is verified"),
        (status = 500, description = "Internal server error")
    ),
    request_body = ForgotParams
)]
#[debug_handler]
async fn verify(State(ctx): State<AppContext>, Path(token): Path<String>) -> Result<Response> {
    let settings = &Settings::from_opt_json(&ctx.config.settings)?;

    let user = users::Model::find_by_verification_token(&ctx.db, &token).await?;

    if user.email_verified_at.is_none() {
        let user = user.into_active_model().verified(&ctx.db).await?;
        tracing::info!(pid = user.pid.to_string(), "user verified");
    }

    format::redirect(format!("https://{}/auth/verification-complete", settings.frontend).as_str())
}

/// Forgot password
///
/// In case the user forgot his password  this endpoints generate a forgot token
/// and send email to the user. In case the email not found in our DB, we are
/// returning a valid request for for security reasons (not exposing users DB
/// list).
#[utoipa::path(
    post,
    path = "/api/auth/forgot",
    tag = "auth",
    responses(
        (status = 200, description = "Reset password email is sent"),
        (status = 500, description = "Internal server error")
    ),
    request_body = ForgotParams
)]
#[debug_handler]
async fn forgot(
    State(ctx): State<AppContext>,
    Json(params): Json<ForgotParams>,
) -> Result<Response> {
    let user = users::Model::find_by_email(&ctx.db, &params.email)
        .await?
        .into_active_model()
        .set_forgot_password_sent(&ctx.db)
        .await?;

    AuthMailer::forgot_password(&ctx, &user).await?;

    format::empty()
}

/// Reset password
///
/// Reset user password by the given parameters
#[utoipa::path(
    post,
    path = "/api/auth/reset",
    tag = "auth",
    responses(
        (status = 200, description = "Password is updated"),
        (status = 500, description = "Internal server error")
    ),
    request_body = ResetParams
)]
#[debug_handler]
async fn reset(State(ctx): State<AppContext>, Json(params): Json<ResetParams>) -> Result<Response> {
    let user = users::Model::find_by_reset_token(&ctx.db, &params.token).await?;

    user.into_active_model()
        .reset_password(&ctx.db, &params.password)
        .await?;

    format::empty()
}

/// Login
///
/// Creates a user login and returns a token
#[utoipa::path(
    post,
    path = "/api/auth/login",
    tag = "auth",
    responses(
        (status = 200, description = "User object", body = LoginResponse),
        (status = 401, description = "Password is incorrect"),
        (status = 403, description = "User email is not verified"),
        (status = 500, description = "Internal server error")
    ),
    request_body = LoginParams
)]
#[debug_handler]
async fn login(
    State(ctx): State<AppContext>,
    cookies: Cookies,
    Json(params): Json<LoginParams>,
) -> Result<Response> {
    let settings = &Settings::from_opt_json(&ctx.config.settings)?;

    let (user, role) = match users::Model::find_by_email_with_role(&ctx.db, &params.email).await {
        Ok((user, role)) => (user, role),
        Err(err) => {
            return match err {
                ModelError::EntityNotFound => {
                    responses::notfound("User with given email is not found")
                }
                _ => responses::internal(),
            }
        }
    };

    if !user.verify_password(&params.password) {
        return responses::unauthorized("Password is incorrect!");
    }

    if user.email_verified_at.is_none() {
        return responses::forbidden("User email is not verified");
    }

    let jwt_secret = ctx.config.get_jwt_config()?;

    let token = user
        .generate_jwt(&jwt_secret.secret, jwt_secret.expiration)
        .or_else(|_| unauthorized("unauthorized!"))?;

    let cookie = Cookie::build(("auth_token", token.clone()))
        .path("/")
        .domain(settings.backend.to_owned())
        .http_only(true)
        .secure(true)
        .same_site(SameSite::None)
        .max_age(time::Duration::seconds(jwt_secret.expiration as i64))
        .build();

    cookies.add(cookie);

    format::json(LoginResponse::new(&user, &role))
}

/// Current Session
///
/// Retrieve current session info
#[utoipa::path(
    get,
    path = "/api/auth/current",
    tag = "auth",
    responses(
        (status = 200, description = "User object", body = CurrentResponse),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error")
    ),
)]
#[debug_handler]
async fn current(auth: auth::JWT, State(ctx): State<AppContext>) -> Result<Response> {
    let (user, role) = users::Model::find_by_pid_with_role(&ctx.db, &auth.claims.pid).await?;
    format::json(CurrentResponse::new(&user, &role))
}

#[debug_handler]
async fn magic_link(
    State(ctx): State<AppContext>,
    Json(params): Json<MagicLinkParams>,
) -> Result<Response> {
    let user = users::Model::find_by_email(&ctx.db, &params.email)
        .await?
        .into_active_model()
        .create_magic_link(&ctx.db)
        .await?;

    AuthMailer::send_magic_link(&ctx, &user).await?;

    format::empty()
}

#[debug_handler]
async fn magic_link_verify(
    Path(token): Path<String>,
    cookies: Cookies,
    State(ctx): State<AppContext>,
) -> Result<Response> {
    let settings = &Settings::from_opt_json(&ctx.config.settings)?;

    let user = users::Model::find_by_magic_token(&ctx.db, &token)
        .await?
        .into_active_model()
        .clear_magic_link(&ctx.db)
        .await?;

    let jwt_secret = ctx.config.get_jwt_config()?;

    let token = user
        .generate_jwt(&jwt_secret.secret, jwt_secret.expiration)
        .or_else(|_| unauthorized("unauthorized!"))?;

    let cookie = Cookie::build(("auth_token", token.clone()))
        .path("/")
        .domain(settings.backend.to_owned())
        .http_only(true)
        .secure(true)
        .same_site(SameSite::None)
        .max_age(time::Duration::seconds(jwt_secret.expiration as i64))
        .build();

    cookies.add(cookie);

    format::redirect(format!("https://{}/auth/login", settings.frontend).as_str())
}

/// Logout
///
/// Logout ending session
#[utoipa::path(
    post,
    path = "/api/auth/logout",
    tag = "auth",
    responses(
        (status = 200, description = "Session ended"),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error")
    ),
)]
#[debug_handler]
async fn logout(
    auth: auth::JWT,
    cookies: Cookies,
    State(ctx): State<AppContext>,
) -> Result<Response> {
    let settings = &Settings::from_opt_json(&ctx.config.settings)?;

    let _ = users::Model::find_by_pid(&ctx.db, &auth.claims.pid).await?;

    let cookie = Cookie::build(("auth_token", ""))
        .path("/")
        .domain(settings.backend.to_owned())
        .http_only(true)
        .secure(true)
        .same_site(SameSite::None)
        .max_age(time::Duration::seconds(0))
        .build();

    cookies.remove(cookie);

    format::empty()
}

/// Delete user account
///
/// Delete user account
#[utoipa::path(
    delete,
    path = "/api/auth/delete",
    tag = "auth",
    responses(
        (status = 200, description = "User deleted"),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error")
    ),
)]
#[debug_handler]
async fn remove(auth: auth::JWT, State(ctx): State<AppContext>) -> Result<Response> {
    users::Model::find_by_pid(&ctx.db, &auth.claims.pid)
        .await?
        .delete(&ctx.db)
        .await?;

    format::empty()
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api/auth")
        .add("/register", openapi(post(register), routes!(register)))
        .add("/verify/{token}", openapi(get(verify), routes!(verify)))
        .add("/login", openapi(post(login), routes!(login)))
        .add("/forgot", openapi(post(forgot), routes!(forgot)))
        .add("/reset", openapi(post(reset), routes!(reset)))
        .add(
            "/resend-verification-mail",
            openapi(
                post(resend_verification_email),
                routes!(resend_verification_email),
            ),
        )
        .add("/current", openapi(get(current), routes!(current)))
        .add("/magic-link", post(magic_link))
        .add("/magic-link/{token}", get(magic_link_verify))
        .add("/logout", openapi(post(logout), routes!(logout)))
        .add("/delete", openapi(delete(remove), routes!(remove)))
}
