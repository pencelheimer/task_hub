use loco_openapi::prelude::ToSchema;
use serde::{Deserialize, Serialize};

use crate::models::{accesses, tasks::AccessLevelEnum};

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct AccessResponse {
    pub accesslevel: AccessLevelEnum,
    pub user_id: i32,
    pub task_id: i32,
}

impl AccessResponse {
    #[must_use]
    pub fn new(access: accesses::Model) -> Self {
        Self {
            accesslevel: access.accesslevel,
            user_id: access.user_id,
            task_id: access.task_id,
        }
    }

    #[must_use]
    pub fn from_vec(accesses: Vec<accesses::Model>) -> Vec<Self> {
        accesses
            .iter()
            .map(|access| Self {
                accesslevel: access.accesslevel,
                user_id: access.user_id,
                task_id: access.task_id,
            })
            .collect()
    }
}
