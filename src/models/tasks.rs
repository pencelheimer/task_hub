use crate::models::_entities::accesses;

pub use super::_entities::{
    sea_orm_active_enums::{AccessLevelEnum, TaskVisibilityEnum},
    tasks::{self, ActiveModel, Entity, Model},
    users,
};

use loco_rs::prelude::*;
use sea_orm::{entity::prelude::*, TransactionTrait};
use serde::{Deserialize, Serialize};
pub type Tasks = Entity;

#[derive(Debug, Validate, Deserialize)]
pub struct Validator {
    #[validate(length(min = 2, message = "Name must be at least 2 characters long."))]
    pub name: String,
}

impl Validatable for ActiveModel {
    fn validator(&self) -> Box<dyn Validate> {
        Box::new(Validator {
            name: self.name.as_ref().to_owned(),
        })
    }
}

#[async_trait::async_trait]
impl ActiveModelBehavior for ActiveModel {
    async fn before_save<C>(mut self, _db: &C, insert: bool) -> std::result::Result<Self, DbErr>
    where
        C: ConnectionTrait,
    {
        if !insert && self.updated_at.is_unchanged() {
            self.updated_at = sea_orm::ActiveValue::Set(chrono::Utc::now().into());
        }

        Ok(self)
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CreateParams {
    pub name: String,
    pub visibility: Option<TaskVisibilityEnum>,
    pub user_pid: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct UpdateParams {
    pub name: Option<String>,
    pub visibility: Option<TaskVisibilityEnum>,
}

// implement your read-oriented logic here
impl Model {
    pub async fn load(db: &DatabaseConnection, id: i32) -> ModelResult<Self> {
        Entity::find_by_id(id)
            .one(db)
            .await?
            .ok_or_else(|| ModelError::EntityNotFound)
    }

    pub async fn list_public(db: &DatabaseConnection) -> ModelResult<Vec<Self>> {
        let tasks = tasks::Entity::find()
            .filter(
                model::query::condition()
                    .eq(tasks::Column::Visibility, TaskVisibilityEnum::Public)
                    .build(),
            )
            .all(db)
            .await?;

        Ok(tasks)
    }

    pub async fn list_for_user(
        db: &DatabaseConnection,
        user_pid: &str,
        asked_by: &str,
    ) -> ModelResult<Vec<Self>> {
        let user = users::Model::find_by_pid(db, user_pid).await?;

        let mut query = tasks::Entity::find()
            .inner_join(accesses::Entity)
            .filter(accesses::Column::UserId.eq(user.id))
            .filter(tasks::Column::Visibility.eq(TaskVisibilityEnum::Public))
            .filter(tasks::Column::Visibility.eq(TaskVisibilityEnum::Paid));

        if user_pid == asked_by {
            query = query.filter(tasks::Column::Visibility.eq(TaskVisibilityEnum::Private));
        }

        let tasks = query.all(db).await?;

        Ok(tasks)
    }

    pub async fn list_for_anon(db: &DatabaseConnection, user_pid: &str) -> ModelResult<Vec<Self>> {
        let user = users::Model::find_by_pid(db, user_pid).await?;

        let query = tasks::Entity::find()
            .inner_join(accesses::Entity)
            .filter(accesses::Column::UserId.eq(user.id))
            .filter(tasks::Column::Visibility.eq(TaskVisibilityEnum::Public))
            .filter(tasks::Column::Visibility.eq(TaskVisibilityEnum::Paid));

        let tasks = query.all(db).await?;

        Ok(tasks)
    }

    pub async fn add(db: &DatabaseConnection, params: CreateParams) -> ModelResult<Self> {
        let user = users::Model::find_by_pid(db, params.user_pid.as_str()).await?;

        let txn = db.begin().await?;

        let task = tasks::ActiveModel {
            name: ActiveValue::set(params.name.clone()),
            visibility: ActiveValue::set(params.visibility.unwrap_or(TaskVisibilityEnum::Private)),
            ..Default::default()
        }
        .insert(&txn)
        .await?;

        accesses::ActiveModel {
            user_id: ActiveValue::set(user.id),
            task_id: ActiveValue::set(task.id),
            accesslevel: ActiveValue::set(AccessLevelEnum::FullAccess),
            ..Default::default()
        }
        .insert(&txn)
        .await?;

        txn.commit().await?;

        Ok(task)
    }
}

// implement your write-oriented logic here
impl ActiveModel {
    pub async fn has_access(
        db: &DatabaseConnection,
        user_pid: String,
        task_id: i32,
        levels: Vec<AccessLevelEnum>,
    ) -> Result<()> {
        let user = users::Model::find_by_pid(db, &user_pid).await?;
        let task = tasks::Model::load(db, task_id).await?;

        let user_access = accesses::Entity::find()
            .filter(accesses::Column::UserId.eq(user.id))
            .filter(accesses::Column::TaskId.eq(task.id))
            .one(db)
            .await?;

        let has_access = match user_access {
            Some(access_level) => levels.contains(&access_level.accesslevel),
            None => false,
        };

        if !has_access {
            unauthorized(format!(
                "user with pid '{}' tried to update task with id {}",
                user.pid, task.id
            ))
        } else {
            Ok(())
        }
    }

    pub async fn update(
        db: &DatabaseConnection,
        params: UpdateParams,
        task_id: i32,
    ) -> Result<Model> {
        let mut active_model = tasks::Model::load(db, task_id).await?.into_active_model();

        if let Some(name) = params.name {
            active_model.name = Set(name);
        }

        if let Some(visibility) = params.visibility {
            active_model.visibility = Set(visibility);
        }

        let task = active_model.update(db).await?;

        Ok(task)
    }

    pub async fn remove(db: &DatabaseConnection, task_id: i32) -> Result<()> {
        tasks::Model::load(db, task_id)
            .await?
            .into_active_model()
            .delete(db)
            .await?;

        Ok(())
    }
}

// implement your custom finders, selectors oriented logic here
impl Entity {}
