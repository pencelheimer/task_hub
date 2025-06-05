use axum::debug_handler;
use loco_openapi::prelude::*;
use loco_rs::prelude::*;

use crate::{
    models::{accesses, tasks},
    views,
};

/// List Task Accesses
///
/// List all Tasks
#[utoipa::path(
    get,
    path = "/api/tasks/access",
    tag = "tasks",
    responses(
        (status = 200, description = "Array of Access objects", body = Vec<views::access::AccessResponse>),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error")
    ),
    params(
        ("id" = i32, Path, description = "Task id"),
    ),
)]
#[debug_handler]
pub async fn list_accesses(
    auth: auth::JWT,
    Path(task_id): Path<i32>,
    State(ctx): State<AppContext>,
) -> Result<Response> {
    tasks::ActiveModel::has_access(
        &ctx.db,
        auth.claims.pid.clone(),
        task_id,
        vec![
            tasks::AccessLevelEnum::FullAccess,
            tasks::AccessLevelEnum::AddUser,
        ],
    )
    .await?;

    let accesses = accesses::Model::list_for_task(&ctx.db, task_id).await?;

    format::json(views::access::AccessResponse::from_vec(accesses))
}

/// Grant Access
///
/// Grant Access to the Task for the User
#[utoipa::path(
    post,
    path = "/api/tasks/access",
    tag = "tasks",
    responses(
        (status = 200, description = "Access is grant"),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error")
    ),
    params(
        ("id" = i32, Path, description = "Task id"),
    ),
    request_body = accesses::GrantParams
)]
#[debug_handler]
pub async fn grant_access(
    auth: auth::JWT,
    Path(task_id): Path<i32>,
    State(ctx): State<AppContext>,
    Json(params): Json<accesses::GrantParams>,
) -> Result<Response> {
    tasks::ActiveModel::has_access(
        &ctx.db,
        auth.claims.pid.clone(),
        task_id,
        vec![
            tasks::AccessLevelEnum::FullAccess,
            tasks::AccessLevelEnum::AddUser,
        ],
    )
    .await?;

    accesses::Model::grant_access(&ctx.db, task_id, params).await?;

    format::empty()
}

/// Update Access
///
/// Update Access to the Task of the User
#[utoipa::path(
    method(put, patch),
    path = "/api/tasks/access",
    tag = "tasks",
    responses(
        (status = 200, description = "Access is updated", body = views::access::AccessResponse),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error")
    ),
    params(
        ("id" = i32, Path, description = "Task id"),
    ),
    request_body = accesses::UpdateParams
)]
#[debug_handler]
pub async fn update_access(
    auth: auth::JWT,
    Path(task_id): Path<i32>,
    State(ctx): State<AppContext>,
    Json(params): Json<accesses::UpdateParams>,
) -> Result<Response> {
    tasks::ActiveModel::has_access(
        &ctx.db,
        auth.claims.pid.clone(),
        task_id,
        vec![
            tasks::AccessLevelEnum::FullAccess,
            tasks::AccessLevelEnum::AddUser,
        ],
    )
    .await?;

    let access = accesses::ActiveModel::update_access(&ctx.db, task_id, params).await?;

    format::json(views::access::AccessResponse::new(access))
}

/// Deny Access
///
/// Deny Access to the Task for the User
#[utoipa::path(
    delete,
    path = "/api/tasks/access",
    tag = "tasks",
    responses(
        (status = 200, description = "Access is Denied"),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error")
    ),
    params(
        ("id" = i32, Path, description = "Task id"),
    ),
    request_body = accesses::DenyParams
)]
#[debug_handler]
pub async fn deny_access(
    auth: auth::JWT,
    Path(task_id): Path<i32>,
    State(ctx): State<AppContext>,
    Json(params): Json<accesses::DenyParams>,
) -> Result<Response> {
    tasks::ActiveModel::has_access(
        &ctx.db,
        auth.claims.pid.clone(),
        task_id,
        vec![
            tasks::AccessLevelEnum::FullAccess,
            tasks::AccessLevelEnum::AddUser,
        ],
    )
    .await?;

    accesses::ActiveModel::deny_access(&ctx.db, task_id, params).await?;

    format::empty()
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("api/tasks/access")
        .add("{id}", openapi(get(list_accesses), routes!(list_accesses)))
        .add("{id}", openapi(post(grant_access), routes!(grant_access)))
        .add("{id}", openapi(delete(deny_access), routes!(deny_access)))
        .add(
            "{id}",
            openapi(patch(update_access), routes!(update_access)),
        )
        .add("{id}", put(update_access))
}
