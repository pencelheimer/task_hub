use loco_openapi::prelude::ToSchema;
use serde::{Deserialize, Serialize};

use crate::models::{_entities::users, roles};

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct GetResponse {
    pub pid: String,
    pub email: String,
    pub name: String,
    pub role: String,
    pub is_verified: bool,
}

impl GetResponse {
    #[must_use]
    pub fn new(user: &users::Model, role: &roles::Model) -> Self {
        Self {
            pid: user.pid.to_string(),
            email: user.email.clone(),
            name: user.name.clone(),
            role: role.name.clone(),
            is_verified: user.email_verified_at.is_some(),
        }
    }
}
