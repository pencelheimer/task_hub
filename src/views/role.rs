use loco_openapi::prelude::ToSchema;
use serde::{Deserialize, Serialize};

use crate::models::roles;

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct GetResponse {
    pub id: i32,
    pub name: String,
}

impl GetResponse {
    #[must_use]
    pub fn new(role: &roles::Model) -> Self {
        Self {
            id: role.id,
            name: role.name.clone(),
        }
    }
}
