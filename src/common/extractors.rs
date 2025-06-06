use axum::extract::{FromRef, OptionalFromRequestParts};
use axum::{extract::FromRequestParts, http::request::Parts};

use loco_rs::controller::extractor::auth;
use loco_rs::{app::AppContext, errors::Error, prelude::*};

use crate::models;

pub struct AdminUser {
    pub jwt: auth::JWT,
    pub user: models::users::Model,
    pub role: models::roles::Model,
}

impl<S> FromRequestParts<S> for AdminUser
where
    AppContext: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = Error;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let jwt = auth::JWT::from_request_parts(parts, state).await?;
        let ctx = match State::<AppContext>::from_request_parts(parts, state).await {
            Ok(state) => state,
            Err(err) => {
                tracing::warn!(message = &err.to_string(), "could not authenticate user",);
                return crate::common::responses::internal();
            }
        };

        let (user, role) =
            models::users::Model::find_by_pid_with_role(&ctx.db, &jwt.claims.pid).await?;

        if role.name != "Admin" {
            return unauthorized("Only users with Admin rights can access this endpoint");
        }

        Ok(Self { jwt, user, role })
    }
}

pub struct OptJWT {
    pub jwt: auth::JWT,
}

impl<S> OptionalFromRequestParts<S> for OptJWT
where
    AppContext: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = Error;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &S,
    ) -> Result<Option<Self>, Self::Rejection> {
        match auth::JWT::from_request_parts(parts, state).await {
            Ok(jwt) => Ok(Some(Self { jwt })),
            Err(_) => Ok(None),
        }
    }
}
