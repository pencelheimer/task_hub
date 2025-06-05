use loco_openapi::prelude::ToSchema;
use serde::{Deserialize, Serialize};

use crate::models::tasks::{self, TaskVisibilityEnum};

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct TaskResponse {
    pub id: i32,
    pub name: String,
    pub visibility: TaskVisibilityEnum,
}

impl TaskResponse {
    #[must_use]
    pub fn new(task: tasks::Model) -> Self {
        Self {
            id: task.id,
            name: task.name.clone(),
            visibility: task.visibility,
        }
    }

    #[must_use]
    pub fn from_vec(tasks: Vec<tasks::Model>) -> Vec<Self> {
        tasks
            .iter()
            .map(|task| Self {
                id: task.id,
                name: task.name.clone(),
                visibility: task.visibility,
            })
            .collect()
    }
}
