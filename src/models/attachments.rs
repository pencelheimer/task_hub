pub use super::_entities::{
    attachments::{ActiveModel, Entity, Model},
    sea_orm_active_enums::AttachmentTypeEnum,
};
use crate::models::{
    self,
    _entities::accesses,
    attachments,
    tasks::{self, AccessLevelEnum},
    users,
};
use axum::body::Bytes;
use axum_typed_multipart::{FieldData, TryFromMultipart};
use loco_rs::prelude::*;
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
pub type Attachments = Entity;

#[async_trait::async_trait]
impl ActiveModelBehavior for ActiveModel {
    async fn before_save<C>(self, _db: &C, insert: bool) -> std::result::Result<Self, DbErr>
    where
        C: ConnectionTrait,
    {
        if !insert && self.updated_at.is_unchanged() {
            let mut this = self;
            this.updated_at = sea_orm::ActiveValue::Set(chrono::Utc::now().into());
            Ok(this)
        } else {
            Ok(self)
        }
    }
}

#[derive(Debug, ToSchema)]
pub struct AttachmentAddParams {
    pub attachment_type: AttachmentTypeEnum,
    pub data: String,
}

#[derive(TryFromMultipart, ToSchema)]
pub struct AttachmentAddForm {
    pub attachment_type: AttachmentTypeEnum,
    pub data: String,

    #[form_data(limit = "10MiB")]
    #[schema(value_type = Vec<u8>, nullable = true)]
    pub file: Option<FieldData<Bytes>>,
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct AttachmentUpdateParams {
    pub data: String,
}

#[derive(TryFromMultipart, ToSchema)]
pub struct AttachmentUpdateForm {
    pub data: String,

    #[form_data(limit = "10MiB")]
    #[schema(value_type = Vec<u8>, nullable = true)]
    pub file: Option<FieldData<Bytes>>,
}

// implement your read-oriented logic here
impl Model {
    pub async fn has_access(
        db: &DatabaseConnection,
        user_pid: &str,
        attachment_id: i32,
        levels: Vec<AccessLevelEnum>,
    ) -> Result<()> {
        let user = users::Model::find_by_pid(db, user_pid).await?;
        let attachment = Model::load(db, attachment_id).await?;

        let user_access = accesses::Entity::find()
            .filter(accesses::Column::UserId.eq(user.id))
            .filter(accesses::Column::TaskId.eq(attachment.task_id))
            .one(db)
            .await?;

        let has_access = match user_access {
            Some(access_level) => levels.contains(&access_level.accesslevel),
            None => false,
        };

        if has_access {
            Ok(())
        } else {
            unauthorized("unauthorized")
        }
    }

    pub async fn load(db: &DatabaseConnection, id: i32) -> ModelResult<Self> {
        Entity::find_by_id(id)
            .one(db)
            .await?
            .ok_or_else(|| ModelError::EntityNotFound)
    }

    pub async fn list_attachments(db: &DatabaseConnection, task_id: i32) -> ModelResult<Vec<Self>> {
        let task = tasks::Model::load(db, task_id).await?;

        let attachments = attachments::Entity::find()
            .filter(models::_entities::attachments::Column::TaskId.eq(task.id))
            .all(db)
            .await?;

        Ok(attachments)
    }

    pub async fn add_attachment(
        db: &DatabaseConnection,
        user_pid: &str,
        task_id: i32,
        params: AttachmentAddParams,
    ) -> ModelResult<Self> {
        let user = users::Model::find_by_pid(db, user_pid).await?;
        let task = tasks::Model::load(db, task_id).await?;

        let attachment = ActiveModel {
            task_id: Set(task.id),
            owner_id: Set(user.id),
            attachment_type: Set(params.attachment_type),
            data: Set(params.data.clone()),
            ..Default::default()
        }
        .insert(db)
        .await?;

        Ok(attachment)
    }
}

// implement your write-oriented logic here
impl ActiveModel {
    pub async fn update_attachment(
        mut self,
        db: &DatabaseConnection,
        user_pid: &str,
        params: AttachmentUpdateParams,
    ) -> ModelResult<Model> {
        let user = users::Model::find_by_pid(db, user_pid).await?;

        self.owner_id = Set(user.id);
        self.data = Set(params.data);

        let attachment = self.update(db).await?;

        Ok(attachment)
    }
}

// implement your custom finders, selectors oriented logic here
impl Entity {}
