use crate::{
    common::settings::Settings,
    mailers::auth::AuthMailer,
    models::{
        _entities::users,
        users::{LoginParams, RegisterParams},
    },
    views::auth::{CurrentResponse, DeleteAllResponse, LoginResponse},
};
use axum::{debug_handler, http::status::StatusCode};
use loco_rs::{controller::ErrorDetail, prelude::*, Error::CustomError};
use tower_cookies::{cookie::SameSite, Cookie, CookieManagerLayer, Cookies};

use regex::Regex;
use serde::{Deserialize, Serialize};
use std::sync::OnceLock;
use time;

pub static EMAIL_DOMAIN_RE: OnceLock<Regex> = OnceLock::new();

#[derive(Debug, Deserialize, Serialize)]
pub struct ForgotParams {
    pub email: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ResetParams {
    pub token: String,
    pub password: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct MagicLinkParams {
    pub email: String,
}

/// Register function creates a new user with the given parameters and sends a
/// welcome email to the user
#[debug_handler]
async fn register(
    State(ctx): State<AppContext>,
    Json(params): Json<RegisterParams>,
) -> Result<Response> {
    let res = users::Model::create_with_password(&ctx.db, &params).await;

    let user = match res {
        Ok(user) => user,
        Err(err) => {
            let message = err.to_string();
            tracing::info!(
                message = &message,
                user_email = &params.email,
                "could not register user",
            );

            return Err(CustomError(
                StatusCode::CONFLICT,
                ErrorDetail {
                    error: Some("Conflict".to_string()),
                    description: Some(message),
                    errors: None,
                },
            ));
        }
    };

    let user = user
        .into_active_model()
        .set_email_verification_sent(&ctx.db)
        .await?;

    AuthMailer::send_welcome(&ctx, &user).await?;

    format::empty()
}

/// Verify register user. if the user not verified his email, he can't login to
/// the system.
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

/// In case the user forgot his password  this endpoints generate a forgot token
/// and send email to the user. In case the email not found in our DB, we are
/// returning a valid request for for security reasons (not exposing users DB
/// list).
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

/// reset user password by the given parameters
#[debug_handler]
async fn reset(State(ctx): State<AppContext>, Json(params): Json<ResetParams>) -> Result<Response> {
    let user = users::Model::find_by_reset_token(&ctx.db, &params.token).await?;

    user.into_active_model()
        .reset_password(&ctx.db, &params.password)
        .await?;

    format::empty()
}

/// Creates a user login and returns a token
#[debug_handler]
async fn login(
    State(ctx): State<AppContext>,
    cookies: Cookies,
    Json(params): Json<LoginParams>,
) -> Result<Response> {
    let settings = &Settings::from_opt_json(&ctx.config.settings)?;

    let user = users::Model::find_by_email(&ctx.db, &params.email).await?;

    if !user.verify_password(&params.password) {
        return unauthorized("unauthorized!");
    }

    if user.email_verified_at.is_none() {
        return Err(CustomError(
            StatusCode::FORBIDDEN,
            ErrorDetail {
                error: Some("email_not_verified".to_string()),
                description: Some("User email is not verified".to_string()),
                errors: None,
            },
        ));
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

    format::json(LoginResponse::new(&user))
}

#[debug_handler]
async fn current(auth: auth::JWT, State(ctx): State<AppContext>) -> Result<Response> {
    let user = users::Model::find_by_pid(&ctx.db, &auth.claims.pid).await?;
    format::json(CurrentResponse::new(&user))
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

    format::empty_json()
}

#[debug_handler]
async fn delete(auth: auth::JWT, State(ctx): State<AppContext>) -> Result<Response> {
    users::Model::find_by_pid(&ctx.db, &auth.claims.pid)
        .await?
        .delete(&ctx.db)
        .await?;

    format::empty()
}
pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api/auth")
        .add("/register", post(register))
        .add("/verify/{token}", get(verify))
        .add("/login", post(login))
        .add("/forgot", post(forgot))
        .add("/reset", post(reset))
        .add("/current", get(current))
        .add("/magic-link", post(magic_link))
        .add("/magic-link/{token}", get(magic_link_verify))
        .add("/logout", post(logout))
        .add("/delete", post(delete))
        .layer(CookieManagerLayer::new())
}
