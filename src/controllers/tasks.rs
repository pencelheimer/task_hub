#![allow(clippy::missing_errors_doc)]
#![allow(clippy::unnecessary_struct_initialization)]
#![allow(clippy::unused_async)]
use axum::debug_handler;
use loco_openapi::prelude::*;
use loco_rs::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{
    common,
    models::{
        accesses, attachments,
        tasks::{self, users, CreateParams, SearchParams, UpdateParams},
    },
    views,
};

#[derive(Clone, Debug, Serialize, Deserialize, ToSchema)]
pub struct Params {
    pub name: String,
    pub visibility: Option<tasks::TaskVisibilityEnum>,
}

#[derive(Clone, Debug, Serialize, Deserialize, ToSchema)]
pub struct FullParams {
    pub task_id: i32,
}

/// Get Full Task
///
/// Get the Task by id
#[utoipa::path(
    post,
    path = "/api/tasks/full",
    tag = "tasks",
    responses(
        (status = 200, description = "Task full object", body = views::task::TaskFullResponse),
        (status = 500, description = "Internal server error")
    ),
    request_body = FullParams
)]
#[debug_handler]
pub async fn get_full(
    State(ctx): State<AppContext>,
    Json(params): Json<FullParams>,
) -> Result<Response> {
    let task = tasks::Model::load(&ctx.db, params.task_id).await?;

    let owner = accesses::Model::find_task_owner(&ctx.db, task.id).await?;

    let (user, role) = users::Model::find_by_id_with_role(&ctx.db, owner.id).await?;

    let attachments = attachments::Model::list_attachments(&ctx.db, task.id).await?;

    format::json(views::task::TaskFullResponse::new(
        task,
        user,
        role,
        attachments,
    ))
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

/// Search Tasks
///
/// Search Tasks
#[utoipa::path(
    post,
    path = "/api/tasks/search",
    tag = "tasks",
    responses(
        (status = 200, description = "Array of Task objects", body = Vec<views::task::TaskResponse>),
        (status = 500, description = "Internal server error")
    ),
    request_body = SearchParams
)]
#[debug_handler]
pub async fn search(
    auth: Option<common::extractors::OptJWT>,
    State(ctx): State<AppContext>,
    Json(params): Json<SearchParams>,
) -> Result<Response> {
    match auth {
        Some(opt_jwt) => format::json(
            tasks::Model::search_for_user(&ctx.db, &opt_jwt.jwt.claims.pid, &params).await?,
        ),
        None => format::json(tasks::Model::search_for_anon(&ctx.db, &params).await?),
    }
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
    request_body = CreateParams
)]
#[debug_handler]
pub async fn add(
    auth: auth::JWT,
    State(ctx): State<AppContext>,
    Json(params): Json<CreateParams>,
) -> Result<Response> {
    let task = tasks::Model::add(&ctx.db, &auth.claims.pid, params).await?;

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
    request_body = UpdateParams
)]
#[debug_handler]
pub async fn update(
    auth: auth::JWT,
    Path(task_id): Path<i32>,
    State(ctx): State<AppContext>,
    Json(params): Json<UpdateParams>,
) -> Result<Response> {
    tasks::Model::has_access(
        &ctx.db,
        &auth.claims.pid,
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
    tasks::Model::has_access(
        &ctx.db,
        &auth.claims.pid,
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
        .add("/search", openapi(post(search), routes!(search)))
        .add("/", openapi(post(add), routes!(add)))
        .add("{id}", openapi(get(get_one), routes!(get_one)))
        .add("/full", openapi(post(get_full), routes!(get_full)))
        .add("{id}", openapi(delete(remove), routes!(remove)))
        .add("{id}", openapi(put(update), routes!(update)))
        .add("{id}", patch(update))
}
