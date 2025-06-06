use loco_openapi::prelude::ToSchema;
use serde::{Deserialize, Serialize};

use crate::{
    models::{attachments, roles, tasks, users},
    views,
};

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct TaskFullResponse {
    pub id: i32,
    pub name: String,
    pub visibility: tasks::TaskVisibilityEnum,
    pub owner: views::user::GetResponse,
    pub attachments: Vec<views::attachment::AttachmentResponse>,
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct TaskResponse {
    pub id: i32,
    pub name: String,
    pub visibility: tasks::TaskVisibilityEnum,
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

impl TaskFullResponse {
    #[must_use]
    pub fn new(
        task: tasks::Model,
        user: users::Model,
        role: roles::Model,
        attachments: Vec<attachments::Model>,
    ) -> Self {
        Self {
            id: task.id,
            name: task.name.clone(),
            visibility: task.visibility,

            owner: views::user::GetResponse::new(&user, &role),
            attachments: views::attachment::AttachmentResponse::from_vec(attachments),
        }
    }
}
