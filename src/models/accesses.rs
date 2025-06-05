use crate::models::{
    _entities::accesses,
    tasks::{self, AccessLevelEnum},
    users,
};

pub use super::_entities::accesses::{ActiveModel, Entity, Model};
use loco_rs::model::{ModelError, ModelResult};
use sea_orm::{entity::prelude::*, ActiveValue::Set, IntoActiveModel};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
pub type Accesses = Entity;

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

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct GrantParams {
    pub email: String,
    pub accesslevel: AccessLevelEnum,
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct UpdateParams {
    pub pid: String,
    pub accesslevel: AccessLevelEnum,
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct DenyParams {
    pub pid: String,
}

// implement your read-oriented logic here
impl Model {
    pub async fn find_by_email(
        db: &DatabaseConnection,
        task_id: i32,
        email: &str,
    ) -> ModelResult<accesses::Model> {
        let user = users::Model::find_by_email(db, email).await?;
        let task = tasks::Model::load(db, task_id).await?;

        let access = accesses::Entity::find()
            .filter(accesses::Column::UserId.eq(user.id))
            .filter(accesses::Column::TaskId.eq(task.id))
            .one(db)
            .await?;

        access.ok_or_else(|| ModelError::EntityNotFound)
    }

    pub async fn find_by_pid(
        db: &DatabaseConnection,
        task_id: i32,
        pid: &str,
    ) -> ModelResult<accesses::Model> {
        let user = users::Model::find_by_pid(db, pid).await?;
        let task = tasks::Model::load(db, task_id).await?;

        let access = accesses::Entity::find()
            .filter(accesses::Column::UserId.eq(user.id))
            .filter(accesses::Column::TaskId.eq(task.id))
            .one(db)
            .await?;

        access.ok_or_else(|| ModelError::EntityNotFound)
    }

    pub async fn list_for_task(db: &DatabaseConnection, task_id: i32) -> ModelResult<Vec<Self>> {
        let task = tasks::Model::load(db, task_id).await?;

        let accesses = accesses::Entity::find()
            .filter(accesses::Column::TaskId.eq(task.id))
            .all(db)
            .await?;

        Ok(accesses)
    }

    pub async fn grant_access(
        db: &DatabaseConnection,
        task_id: i32,
        params: GrantParams,
    ) -> ModelResult<accesses::Model> {
        let user = users::Model::find_by_email(db, &params.email).await?;
        let task = tasks::Model::load(db, task_id).await?;

        let access = accesses::ActiveModel {
            user_id: Set(user.id),
            task_id: Set(task.id),
            accesslevel: Set(params.accesslevel),
            ..Default::default()
        }
        .insert(db)
        .await?;

        Ok(access)
    }
}

// implement your write-oriented logic here
impl ActiveModel {
    pub async fn update_access(
        db: &DatabaseConnection,
        task_id: i32,
        params: UpdateParams,
    ) -> ModelResult<accesses::Model> {
        let mut active_model = accesses::Model::find_by_pid(db, task_id, &params.pid)
            .await?
            .into_active_model();

        active_model.accesslevel = Set(params.accesslevel);

        let access = active_model.update(db).await?;

        Ok(access)
    }

    pub async fn deny_access(
        db: &DatabaseConnection,
        task_id: i32,
        params: DenyParams,
    ) -> ModelResult<()> {
        accesses::Model::find_by_pid(db, task_id, &params.pid)
            .await?
            .into_active_model()
            .delete(db)
            .await?;

        Ok(())
    }
}

// implement your custom finders, selectors oriented logic here
impl Entity {}
