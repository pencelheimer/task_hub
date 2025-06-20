use async_trait::async_trait;
use axum::extract::DefaultBodyLimit;
use loco_openapi::prelude::*;
use loco_rs::{
    app::{AppContext, Hooks, Initializer},
    bgworker::{BackgroundWorker, Queue},
    boot::{create_app, BootResult, StartMode},
    config::Config,
    controller::AppRoutes,
    db::{self, truncate_table},
    environment::Environment,
    storage,
    task::Tasks,
    Result,
};
use migration::Migrator;
use std::path::Path;

use tower_cookies::CookieManagerLayer;

#[allow(unused_imports)]
use crate::{controllers, models::_entities::users, tasks, workers::downloader::DownloadWorker};
use crate::{initializers, models::roles};

pub struct App;
#[async_trait]
impl Hooks for App {
    fn app_name() -> &'static str {
        env!("CARGO_CRATE_NAME")
    }

    fn app_version() -> String {
        format!(
            "{} ({})",
            env!("CARGO_PKG_VERSION"),
            option_env!("BUILD_SHA")
                .or(option_env!("GITHUB_SHA"))
                .unwrap_or("dev")
        )
    }

    async fn boot(
        mode: StartMode,
        environment: &Environment,
        config: Config,
    ) -> Result<BootResult> {
        create_app::<Self, Migrator>(mode, environment, config).await
    }

    async fn initializers(_ctx: &AppContext) -> Result<Vec<Box<dyn Initializer>>> {
        Ok(vec![
            Box::new(loco_openapi::OpenapiInitializerWithSetup::new(
                |ctx| {
                    #[derive(OpenApi)]
                    #[openapi(
                        modifiers(&SecurityAddon),
                        info(
                            title = "TaskHub",
                            description = "The best Task Managment Solution EVER"
                        )
                    )]
                    struct ApiDoc;
                    set_jwt_location(ctx.into());

                    ApiDoc::openapi()
                },
                None,
            )),
            Box::new(initializers::axum_session::AxumSessionInitializer),
            Box::new(initializers::oauth2::OAuth2StoreInitializer),
        ])
    }

    fn routes(_ctx: &AppContext) -> AppRoutes {
        AppRoutes::with_default_routes() // controller routes below
            .add_route(controllers::attachments::routes())
            .add_route(controllers::users::routes())
            .add_route(controllers::tasks::routes())
            .add_route(controllers::accesses::routes())
            .add_route(controllers::roles::routes())
            .add_route(controllers::auth::routes())
            .add_route(controllers::oauth2::routes())
    }

    async fn after_routes(router: axum::Router, _ctx: &AppContext) -> Result<axum::Router> {
        let router = router
            .layer(CookieManagerLayer::new())
            .layer(DefaultBodyLimit::max(1024 * 1024 * 10));

        Ok(router)
    }

    async fn after_context(ctx: AppContext) -> Result<AppContext> {
        Ok(AppContext {
            storage: storage::Storage::single(storage::drivers::mem::new()).into(),
            ..ctx
        })
    }

    async fn connect_workers(ctx: &AppContext, queue: &Queue) -> Result<()> {
        queue.register(DownloadWorker::build(ctx)).await?;
        Ok(())
    }

    #[allow(unused_variables)]
    fn register_tasks(tasks: &mut Tasks) {
        // tasks-inject (do not remove)
    }
    async fn truncate(ctx: &AppContext) -> Result<()> {
        truncate_table(&ctx.db, users::Entity).await?;
        Ok(())
    }
    async fn seed(ctx: &AppContext, base: &Path) -> Result<()> {
        db::seed::<roles::ActiveModel>(&ctx.db, &base.join("roles.yaml").display().to_string())
            .await?;

        db::seed::<users::ActiveModel>(&ctx.db, &base.join("users.yaml").display().to_string())
            .await?;

        Ok(())
    }
}
