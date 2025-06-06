#![allow(clippy::missing_errors_doc)]
#![allow(clippy::unnecessary_struct_initialization)]
#![allow(clippy::unused_async)]
use std::path::PathBuf;

use axum::debug_handler;

use axum_typed_multipart::TypedMultipart;
use loco_openapi::prelude::*;
use loco_rs::prelude::*;

use crate::{
    common::responses,
    models::{
        attachments::{self, *},
        tasks,
    },
    views::attachment::*,
};

/// List Attachments
///
/// List Attachments of the Task
#[utoipa::path(
    get,
    path = "/api/tasks/attachments/{id}",
    tag = "attachments",
    responses(
        (status = 200, description = "Array of Attachment objects", body = Vec<AttachmentResponse>),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error")
    ),
    params(
        ("id" = i32, Path, description = "Task id"),
    ),
)]
#[debug_handler]
pub async fn list(
    auth: auth::JWT,
    Path(task_id): Path<i32>,
    State(ctx): State<AppContext>,
) -> Result<Response> {
    tasks::Model::has_access(
        &ctx.db,
        &auth.claims.pid,
        task_id,
        vec![
            tasks::AccessLevelEnum::FullAccess,
            tasks::AccessLevelEnum::AddUser,
            tasks::AccessLevelEnum::Edit,
            tasks::AccessLevelEnum::AddSolution,
            tasks::AccessLevelEnum::View,
        ],
    )
    .await?;

    let attachment = attachments::Model::list_attachments(&ctx.db, task_id).await?;

    format::json(AttachmentResponse::from_vec(attachment))
}

/// Add Attachment
///
/// Add Attachment, either by providing a file (multipart)
/// or by providing structured data (via JSON in a multipart field)
#[utoipa::path(
    post,
    path = "/api/tasks/attachments/{id}",
    tag = "attachments",
    request_body(
        content_type = "multipart/form-data",
        content = AttachmentAddForm
    ),
    responses(
        (status = 200, description = "Attachment is added/uploaded", body = AttachmentResponse),
        (status = 400, description = "Bad Request"),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error")
    ),
    params(
        ("id" = i32, Path, description = "Task id"),
    ),
)]
#[debug_handler]
pub async fn add(
    auth: auth::JWT,
    Path(task_id): Path<i32>,
    State(ctx): State<AppContext>,
    TypedMultipart(form): TypedMultipart<attachments::AttachmentAddForm>,
) -> Result<Response> {
    tasks::Model::has_access(
        &ctx.db,
        &auth.claims.pid,
        task_id,
        vec![
            tasks::AccessLevelEnum::FullAccess,
            tasks::AccessLevelEnum::AddUser,
            tasks::AccessLevelEnum::Edit,
        ],
    )
    .await?;

    match form.attachment_type {
        AttachmentTypeEnum::File => {
            let (file_name, content) = if let Some(field) = form.file {
                let fname = field
                    .metadata
                    .file_name
                    .ok_or_else(|| Error::BadRequest("File field missing filename".into()))?
                    .to_string();

                let bytes = field.contents;

                (fname, bytes)
            } else {
                return Err(Error::BadRequest(
                    "File content is required for provided Attachment".into(),
                ));
            };

            let params = AttachmentAddParams {
                attachment_type: form.attachment_type,
                data: file_name.clone(),
            };

            let attachment =
                attachments::Model::add_attachment(&ctx.db, &auth.claims.pid, task_id, params)
                    .await?;

            let path = PathBuf::from(attachment.id.to_string()).join(&file_name);

            ctx.storage
                .as_ref()
                .upload(path.as_path(), &content)
                .await?;

            format::json(AttachmentResponse::new(attachment))
        }
        _ => {
            if form.file.is_some() {
                return Err(Error::BadRequest(
                    "File upload not expected for provided Attachment".into(),
                ));
            }

            let params = AttachmentAddParams {
                attachment_type: form.attachment_type,
                data: form.data,
            };

            let attachment =
                attachments::Model::add_attachment(&ctx.db, &auth.claims.pid, task_id, params)
                    .await?;

            format::json(AttachmentResponse::new(attachment))
        }
    }
}

/// Update Attachment
///
/// Update Attachment, either by providing a new file (multipart)
/// or by providing updated structured data (via JSON in a multipart field)
#[utoipa::path(
    method(put, patch),
    path = "/api/tasks/attachments/{id}",
    tag = "attachments",
    request_body(
        content_type = "multipart/form-data",
        content = AttachmentUpdateForm // A new form type for update
    ),
    responses(
        (status = 200, description = "Attachment is updated", body = AttachmentResponse),
        (status = 400, description = "Bad Request"),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error")
    ),
    params(
        ("id" = i32, Path, description = "Attachment id"),
    ),
)]
#[debug_handler]
pub async fn update(
    auth: auth::JWT,
    Path(attachment_id): Path<i32>,
    State(ctx): State<AppContext>,
    TypedMultipart(form): TypedMultipart<attachments::AttachmentUpdateForm>, // Use a new form type
) -> Result<Response> {
    let attachment = match attachments::Model::load(&ctx.db, attachment_id).await {
        Ok(att) => att,
        Err(ModelError::EntityNotFound) => return responses::notfound("Attachment not found."),
        _ => return responses::internal(),
    };

    tasks::Model::has_access(
        &ctx.db,
        &auth.claims.pid,
        attachment.task_id,
        vec![
            tasks::AccessLevelEnum::FullAccess,
            tasks::AccessLevelEnum::AddUser,
            tasks::AccessLevelEnum::Edit,
        ],
    )
    .await?;

    let data = match attachment.attachment_type {
        AttachmentTypeEnum::File => match form.file {
            Some(field) => {
                let file_name = field
                    .metadata
                    .file_name
                    .ok_or_else(|| Error::BadRequest("File field missing filename".into()))?
                    .to_string();

                let bytes = field.contents;

                let old_path = PathBuf::from(attachment.id.to_string()).join(&attachment.data);
                ctx.storage.as_ref().delete(old_path.as_path()).await.ok();

                let path = PathBuf::from(attachment.id.to_string()).join(&file_name);
                ctx.storage.as_ref().upload(path.as_path(), &bytes).await?;

                file_name
            }
            None => return responses::bad_request("File content is required for such Attachment"),
        },
        _ => {
            if form.file.is_some() {
                return responses::bad_request("File content is not expected for such Attachment");
            }

            form.data
        }
    };

    let params = AttachmentUpdateParams { data };

    let updated_attachment = attachment
        .into_active_model()
        .update_attachment(&ctx.db, &auth.claims.pid, params)
        .await?;

    format::json(AttachmentResponse::new(updated_attachment))
}

/// Remove Attachment
///
/// Remove Attachment
#[utoipa::path(
    delete,
    path = "/api/tasks/attachments/{id}",
    tag = "attachments",
    responses(
        (status = 200, description = "Attachment is deleted"),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error")
    ),
    params(
        ("id" = i32, Path, description = "Attachment id"),
    ),
)]
#[debug_handler]
pub async fn remove(
    auth: auth::JWT,
    Path(attachment_id): Path<i32>,
    State(ctx): State<AppContext>,
) -> Result<Response> {
    attachments::Model::has_access(
        &ctx.db,
        &auth.claims.pid,
        attachment_id,
        vec![
            tasks::AccessLevelEnum::FullAccess,
            tasks::AccessLevelEnum::AddUser,
            tasks::AccessLevelEnum::Edit,
        ],
    )
    .await?;

    let attachment = match attachments::Model::load(&ctx.db, attachment_id).await {
        Ok(att) => att,
        Err(ModelError::EntityNotFound) => return responses::notfound("Attachment not found."),
        _ => return responses::internal(),
    };

    if let AttachmentTypeEnum::File = attachment.attachment_type {
        let path = PathBuf::from(attachment.id.to_string()).join(&attachment.data);
        ctx.storage.as_ref().delete(path.as_path()).await.ok();
    }

    attachment.into_active_model().delete(&ctx.db).await?;

    format::empty()
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("api/tasks/attachments/")
        .add("{id}", openapi(get(list), routes!(list)))
        .add("{id}", openapi(post(add), routes!(add)))
        .add("{id}", openapi(patch(update), routes!(update)))
        .add("{id}", put(update))
        .add("{id}", openapi(delete(remove), routes!(remove)))
}
