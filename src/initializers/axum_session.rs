use async_trait::async_trait;
use axum::Router as AxumRouter;
use axum_session::SessionConfig;
use loco_rs::prelude::*;
use tower_cookies::cookie::SameSite;

use crate::common::settings::Settings;

pub struct AxumSessionInitializer;

#[async_trait]
impl Initializer for AxumSessionInitializer {
    fn name(&self) -> String {
        "axum-session".to_string()
    }

    async fn after_routes(&self, router: AxumRouter, ctx: &AppContext) -> Result<AxumRouter> {
        let settings = &Settings::from_opt_json(&ctx.config.settings)?;

        let session_config = SessionConfig::default()
            .with_table_name("sessions_table")
            .with_cookie_domain(settings.backend.to_owned())
            .with_secure(true)
            .with_cookie_same_site(SameSite::None);

        let session_store =
            axum_session::SessionStore::<axum_session::SessionNullPool>::new(None, session_config)
                .await
                .unwrap();

        let router = router.layer(axum_session::SessionLayer::new(session_store));

        Ok(router)
    }
}
