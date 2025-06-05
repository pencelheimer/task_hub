#![allow(clippy::missing_errors_doc)]
#![allow(clippy::unnecessary_struct_initialization)]
#![allow(clippy::unused_async)]
use axum::debug_handler;
use loco_openapi::prelude::*;
use loco_rs::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{
    models::tasks::{self, CreateParams, UpdateParams},
    views,
};

#[derive(Clone, Debug, Serialize, Deserialize, ToSchema)]
pub struct Params {
    pub name: String,
    pub visibility: Option<tasks::TaskVisibilityEnum>,
}

/// Get Task
///
/// Get the Task by id
#[utoipa::path(
    get,
    path = "/api/tasks",
    tag = "tasks",
    responses(
        (status = 200, description = "Task object", body = views::task::TaskResponse),
        (status = 500, description = "Internal server error")
    ),
    params(
        ("id" = i32, Path, description = "Task id"),
    )
)]
#[debug_handler]
pub async fn get_one(Path(id): Path<i32>, State(ctx): State<AppContext>) -> Result<Response> {
    let task = tasks::Model::load(&ctx.db, id).await?;
    format::json(views::task::TaskResponse::new(task))
}

/// List Tasks
///
/// List all Tasks
#[utoipa::path(
    get,
    path = "/api/tasks/list",
    tag = "tasks",
    responses(
        (status = 200, description = "Array of Task objects", body = Vec<views::task::TaskResponse>),
        (status = 500, description = "Internal server error")
    ),
)]
#[debug_handler]
pub async fn list(State(ctx): State<AppContext>) -> Result<Response> {
    format::json(tasks::Model::list_public(&ctx.db).await?)
}

/// Create Task
///
/// Create new Task
#[utoipa::path(
    post,
    path = "/api/tasks",
    tag = "tasks",
    responses(
        (status = 200, description = "Task created", body = views::task::TaskResponse),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error")
    ),
    request_body = Params
)]
#[debug_handler]
pub async fn add(
    auth: auth::JWT,
    State(ctx): State<AppContext>,
    Json(params): Json<Params>,
) -> Result<Response> {
    let params = CreateParams {
        name: params.name,
        visibility: params.visibility,
        user_pid: auth.claims.pid,
    };

    let task = tasks::Model::add(&ctx.db, params).await?;

    format::json(views::task::TaskResponse::new(task))
}

/// Update Task
///
/// Create new Task
#[utoipa::path(
    method(put, patch),
    path = "/api/tasks",
    tag = "tasks",
    responses(
        (status = 200, description = "Task updated", body = views::task::TaskResponse),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error")
    ),
    params(
        ("id" = i32, Path, description = "Task id"),
    ),
    request_body = Params
)]
#[debug_handler]
pub async fn update(
    auth: auth::JWT,
    Path(task_id): Path<i32>,
    State(ctx): State<AppContext>,
    Json(params): Json<UpdateParams>,
) -> Result<Response> {
    tasks::ActiveModel::has_access(
        &ctx.db,
        auth.claims.pid.clone(),
        task_id,
        vec![
            tasks::AccessLevelEnum::FullAccess,
            tasks::AccessLevelEnum::Edit,
            tasks::AccessLevelEnum::AddUser,
        ],
    )
    .await?;

    let task = tasks::ActiveModel::update(&ctx.db, params, task_id).await?;

    format::json(task)
}

/// Remove Task
///
/// Remove existing Task
#[utoipa::path(
    delete,
    path = "/api/tasks",
    tag = "tasks",
    responses(
        (status = 200, description = "Task removed"),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error")
    ),
    params(
        ("id" = i32, Path, description = "Task id"),
    ),
)]
#[debug_handler]
pub async fn remove(
    auth: auth::JWT,
    Path(task_id): Path<i32>,
    State(ctx): State<AppContext>,
) -> Result<Response> {
    tasks::ActiveModel::has_access(
        &ctx.db,
        auth.claims.pid.clone(),
        task_id,
        vec![tasks::AccessLevelEnum::FullAccess],
    )
    .await?;

    tasks::ActiveModel::remove(&ctx.db, task_id).await?;

    format::empty()
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("api/tasks/")
        .add("/list", openapi(get(list), routes!(list)))
        .add("/", openapi(post(add), routes!(add)))
        .add("{id}", openapi(get(get_one), routes!(get_one)))
        .add("{id}", openapi(delete(remove), routes!(remove)))
        .add("{id}", openapi(put(update), routes!(update)))
        .add("{id}", patch(update))
}
