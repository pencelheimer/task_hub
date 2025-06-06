use axum::{extract::Query, Extension};
use axum_session::{SameSite, Session, SessionNullPool};
use loco_oauth2::{
    controllers::{
        middleware::OAuth2CookieUser,
        oauth2::{callback_jwt, google_callback_cookie, AuthParams},
    },
    OAuth2ClientStore,
};
use loco_rs::prelude::*;
use tower_cookies::{Cookie, Cookies};

use crate::{
    common::settings::Settings,
    models::{
        o_auth2_sessions,
        users::{self, OAuth2UserProfile},
    },
};

async fn protected(
    State(ctx): State<AppContext>,
    // Extract the user from the Cookie via middleware
    user: OAuth2CookieUser<OAuth2UserProfile, users::Model, o_auth2_sessions::Model>,
) -> Result<Response> {
    let user: &users::Model = user.as_ref();
    let jwt_secret = ctx.config.get_jwt_config()?;
    // Generate a JWT token
    let token = user
        .generate_jwt(&jwt_secret.secret, jwt_secret.expiration)
        .or_else(|_| unauthorized("unauthorized!"))?;
    // Return the user and the token in JSON format
    format::json(user)
}

pub async fn google_authorization_url(
    session: Session<SessionNullPool>,
    Extension(oauth2_store): Extension<OAuth2ClientStore>,
) -> Result<String> {
    let mut client = oauth2_store
        .get_authorization_code_client("google")
        .await
        .map_err(|e| {
            tracing::error!("Error getting client: {:?}", e);
            Error::InternalServerError
        })?;

    let (auth_url, csrf_token) = client.get_authorization_url();
    session.set("CSRF_TOKEN", csrf_token.secret().to_owned());

    session.update();

    drop(client);

    Ok(auth_url.to_string())
}

pub async fn google_callback_jwt(
    State(ctx): State<AppContext>,
    session: Session<SessionNullPool>,
    cookies: Cookies,
    Query(params): Query<AuthParams>,
    Extension(oauth2_store): Extension<OAuth2ClientStore>,
) -> Result<impl IntoResponse> {
    let settings = &Settings::from_opt_json(&ctx.config.settings)?;

    let mut client = oauth2_store
        .get_authorization_code_client("google")
        .await
        .map_err(|e| {
            tracing::error!("Error getting client: {:?}", e);
            Error::InternalServerError
        })?;

    let jwt_secret = ctx.config.get_jwt_config()?;

    let user = callback_jwt::<
        OAuth2UserProfile,
        users::Model,
        o_auth2_sessions::Model,
        SessionNullPool,
    >(&ctx, session, params, &mut client)
    .await?;

    drop(client);

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

    format::redirect(format!("https://{}/profile", settings.frontend).as_str())
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("api/oauth2")
        .add("/google", get(google_authorization_url))
        // Route for the Cookie callback
        .add(
            "/google/callback/cookie",
            get(google_callback_cookie::<
                OAuth2UserProfile,
                users::Model,
                o_auth2_sessions::Model,
                SessionNullPool,
            >),
        )
        // Route for the JWT callback
        .add("/google/callback/jwt", get(google_callback_jwt))
        .add("/protected", get(protected))
}
