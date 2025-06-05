use crate::models::{o_auth2_sessions, users, users::OAuth2UserProfile};
use axum::Extension;
use axum_session::Session;
use axum_session::SessionNullPool;
use loco_oauth2::controllers::oauth2::get_authorization_url;
use loco_oauth2::OAuth2ClientStore;

use loco_openapi::prelude::*;
use loco_rs::prelude::*;

use loco_oauth2::controllers::oauth2::google_callback_jwt;

/// Redirect to OAuth2
///
/// Redirect handler to the authorization URL for the `OAuth2` flow
/// This will redirect the user to the `OAuth2` provider's login page
/// and then to the callback URL
#[utoipa::path(
    get,
    path = "/api/oauth2/google",
    tag = "oauth2",
    responses(
        (status = 307, description = "Redirect to OAuth2 URL"),
        (status = 500, description = "Internal server error")
    ),
)]
pub async fn redirect_to_google_authorization_url(
    session: Session<SessionNullPool>,
    Extension(oauth2_store): Extension<OAuth2ClientStore>,
) -> Result<impl IntoResponse> {
    // Get the `google` Authorization Code Grant client from the `OAuth2ClientStore`
    let mut client = oauth2_store
        .get_authorization_code_client("google")
        .await
        .map_err(|e| {
            tracing::error!("Error getting client: {:?}", e);
            Error::InternalServerError
        })?;
    // Get the authorization URL and save the csrf token in the session
    let auth_url = get_authorization_url(session, &mut client).await;
    drop(client);
    Ok(axum::response::Redirect::temporary(&auth_url).into_response())
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("api/oauth2/google")
        .add(
            "/",
            openapi(
                get(redirect_to_google_authorization_url),
                routes!(redirect_to_google_authorization_url),
            ),
        )
        // Route for the JWT callback
        .add(
            "/callback/jwt",
            get(google_callback_jwt::<
                OAuth2UserProfile,
                users::Model,
                o_auth2_sessions::Model,
                SessionNullPool,
            >),
        )
}
