use loco_openapi::prelude::ToSchema;
use serde::{Deserialize, Serialize};

use crate::models::{_entities::users, roles};

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct LoginResponse {
    pub pid: String,
    pub name: String,
    pub is_verified: bool,
    pub role: String,
}

impl LoginResponse {
    #[must_use]
    pub fn new(user: &users::Model, role: &roles::Model) -> Self {
        Self {
            pid: user.pid.to_string(),
            name: user.name.clone(),
            is_verified: user.email_verified_at.is_some(),
            role: role.name.clone(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct CurrentResponse {
    pub pid: String,
    pub name: String,
    pub email: String,
    pub role: String,
}

impl CurrentResponse {
    #[must_use]
    pub fn new(user: &users::Model, role: &roles::Model) -> Self {
        Self {
            pid: user.pid.to_string(),
            name: user.name.clone(),
            email: user.email.clone(),
            role: role.name.clone(),
        }
    }
}
