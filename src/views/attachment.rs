use loco_openapi::prelude::ToSchema;
use serde::{Deserialize, Serialize};

use crate::models::attachments::*;

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct AttachmentResponse {
    pub task_id: i32,
    pub attachment_type: AttachmentTypeEnum,
    pub data: String,
}

impl AttachmentResponse {
    #[must_use]
    pub fn new(attachment: Model) -> Self {
        Self {
            task_id: attachment.task_id,
            attachment_type: attachment.attachment_type,
            data: attachment.data.clone(),
        }
    }

    #[must_use]
    pub fn from_vec(attachment: Vec<Model>) -> Vec<Self> {
        attachment
            .iter()
            .map(|attachment| Self {
                task_id: attachment.task_id,
                attachment_type: attachment.attachment_type,
                data: attachment.data.clone(),
            })
            .collect()
    }
}
