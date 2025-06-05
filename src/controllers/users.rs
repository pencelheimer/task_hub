#![allow(clippy::missing_errors_doc)]
#![allow(clippy::unnecessary_struct_initialization)]
#![allow(clippy::unused_async)]
use axum::debug_handler;
use loco_openapi::prelude::*;
use loco_rs::prelude::*;

use crate::{
    models::{tasks, users},
    views::{self, user::GetResponse},
};

/// Get User
///
/// Get the user info by pid
#[utoipa::path(
    get,
    path = "/api/user",
    tag = "users",
    responses(
        (status = 200, description = "User object", body = GetResponse),
        (status = 500, description = "Internal server error")
    ),
    params(
        ("pid" = String, Path, description = "User's pid"),
    ),
)]
#[debug_handler]
pub async fn get_one(Path(pid): Path<String>, State(ctx): State<AppContext>) -> Result<Response> {
    let (user, role) = users::Model::find_by_pid_with_role(&ctx.db, &pid).await?;

    format::json(GetResponse::new(&user, &role))
}

/// Get Me
///
/// Get the current user info
#[utoipa::path(
    get,
    path = "/api/user/me",
    tag = "users",
    responses(
        (status = 200, description = "User object", body = GetResponse),
        (status = 401, description = "Unathorised"),
        (status = 500, description = "Internal server error")
    ),
)]
#[debug_handler]
pub async fn get_me(auth: auth::JWT, State(ctx): State<AppContext>) -> Result<Response> {
    let (user, role) = users::Model::find_by_pid_with_role(&ctx.db, &auth.claims.pid).await?;

    format::json(GetResponse::new(&user, &role))
}

/// User's Tasks
///
/// Get the list of user's tasks by pid
#[utoipa::path(
    get,
    path = "/api/user/tasks",
    tag = "users",
    responses(
        (status = 200, description = "Array of Task objects", body = Vec<views::task::TaskResponse>),
        (status = 500, description = "Internal server error")
    ),
    params(
        ("pid" = String, Path, description = "User's pid"),
    ),
)]
#[debug_handler]
pub async fn tasks(Path(pid): Path<String>, State(ctx): State<AppContext>) -> Result<Response> {
    let tasks = tasks::Model::list_for_anon(&ctx.db, &pid).await?;

    format::json(views::task::TaskResponse::from_vec(tasks))
}

/// My Tasks
///
/// Get the list of current user's tasks
#[utoipa::path(
    get,
    path = "/api/user/tasks/me",
    tag = "users",
    responses(
        (status = 200, description = "Array of Task objects", body = Vec<views::task::TaskResponse>),
        (status = 401, description = "Unathorised"),
        (status = 500, description = "Internal server error")
    ),
)]
#[debug_handler]
pub async fn tasks_me(auth: auth::JWT, State(ctx): State<AppContext>) -> Result<Response> {
    let tasks = tasks::Model::list_for_user(&ctx.db, &auth.claims.pid, &auth.claims.pid).await?;

    format::json(views::task::TaskResponse::from_vec(tasks))
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("api/user/")
        .add("/me", openapi(get(get_me), routes!(get_me)))
        .add("/{pid}", openapi(get(get_one), routes!(get_one)))
        .add("tasks/me", openapi(get(tasks_me), routes!(tasks_me)))
        .add("tasks/{pid}", openapi(get(tasks), routes!(tasks)))
}
