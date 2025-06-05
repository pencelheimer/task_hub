#![allow(clippy::missing_errors_doc)]
#![allow(clippy::unnecessary_struct_initialization)]
#![allow(clippy::unused_async)]
use axum::debug_handler;
use loco_openapi::prelude::*;
use loco_rs::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{
    common,
    models::_entities::roles::{ActiveModel, Entity, Model},
    views,
};

#[derive(Clone, Debug, Serialize, Deserialize, ToSchema)]
pub struct Params {
    pub name: String,
}

impl Params {
    fn update(&self, item: &mut ActiveModel) {
        item.name = Set(self.name.clone());
    }
}

async fn load_item(ctx: &AppContext, id: i32) -> Result<Model> {
    let item = Entity::find_by_id(id).one(&ctx.db).await?;
    item.ok_or_else(|| Error::NotFound)
}

/// List
///
/// List all Roles in the DB
#[utoipa::path(
    get,
    path = "/api/roles",
    tag = "roles",
    responses(
        (status = 200, description = "List of Role Objects", body = Vec<views::role::GetResponse>),
        (status = 500, description = "Internal server error")
    ),
    request_body = Params
)]
#[debug_handler]
pub async fn list(State(ctx): State<AppContext>) -> Result<Response> {
    format::json(Entity::find().all(&ctx.db).await?)
}

/// Create
///
/// Create new Role
#[utoipa::path(
    post,
    path = "/api/roles",
    tag = "roles",
    responses(
        (status = 200, description = "Role created", body = views::role::GetResponse),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error")
    ),
    request_body = Params
)]
#[debug_handler]
pub async fn add(
    _admin: common::extractors::AdminUser,
    State(ctx): State<AppContext>,
    Json(params): Json<Params>,
) -> Result<Response> {
    let mut item = ActiveModel {
        ..Default::default()
    };
    params.update(&mut item);
    let item = item.insert(&ctx.db).await?;
    format::json(item)
}

/// Update
///
/// Update the Role in the DB
#[utoipa::path(
    method(put, patch),
    path = "/api/roles/update",
    tag = "roles",
    responses(
        (status = 200, description = "Role updated", body = Params),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error")
    ),
    params(
        ("id" = i32, Path, description = "Role id"),
    ),
)]
#[debug_handler]
pub async fn update(
    _admin: common::extractors::AdminUser,
    Path(id): Path<i32>,
    State(ctx): State<AppContext>,
    Json(params): Json<Params>,
) -> Result<Response> {
    let item = load_item(&ctx, id).await?;
    let mut item = item.into_active_model();
    params.update(&mut item);
    let item = item.update(&ctx.db).await?;
    format::json(item)
}

/// Delete
///
/// Delete the Role from the DB
#[utoipa::path(
    delete,
    path = "/api/roles/delete",
    tag = "roles",
    responses(
        (status = 200, description = "Role deleted"),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error")
    ),
    params(
        ("id" = i32, Path, description = "Role id"),
    )
)]
#[debug_handler]
pub async fn remove(
    _admin: common::extractors::AdminUser,
    Path(id): Path<i32>,
    State(ctx): State<AppContext>,
) -> Result<Response> {
    load_item(&ctx, id).await?.delete(&ctx.db).await?;
    format::empty()
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("api/roles/")
        .add("/", openapi(get(list), routes!(list)))
        .add("/", openapi(post(add), routes!(add)))
        .add("{id}", openapi(delete(remove), routes!(remove)))
        .add("{id}", openapi(put(update), routes!(update)))
        .add("{id}", patch(update))
}
