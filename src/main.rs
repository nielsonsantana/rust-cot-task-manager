mod locale_middleware;
mod api_auth;
mod api_tasks;
mod cqrs;
mod models;
mod migrations;
mod admin;
mod auth_extractor;

rust_i18n::i18n!("locales", fallback = "en");

use cot::Template;
use async_trait::async_trait;
use cot::admin::{AdminApp, AdminModelManager, DefaultAdminModelManager};
use cot::openapi::swagger_ui::SwaggerUi;
use cot::router::method::openapi::{api_post, api_get, api_delete, api_patch};
use cot::auth::db::{DatabaseUser, DatabaseUserApp};
use cot::cli::CliMetadata;
use cot::db::migrations::SyncDynMigration;
use cot::middleware::{AuthMiddleware, LiveReloadMiddleware, SessionMiddleware};
use cot::project::{MiddlewareContext, RegisterAppsContext, RootHandler, RootHandlerBuilder};
use cot::router::{Route, Router, Urls};
use cot::html::Html;
use cot::static_files::StaticFilesMiddleware;
use cot::session::db::SessionApp;
use cot::{App, AppBuilder, Project, ProjectContext};

use locale_middleware::LocaleMiddleware;

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate<'a> {
    #[warn(dead_code)]
    urls: &'a Urls,
}

async fn index(urls: Urls) -> cot::Result<Html> {
    let template = IndexTemplate { urls: &urls };

    Ok(Html::new(template.render()?))
}

struct TaskManagerApp;

#[async_trait]
impl App for TaskManagerApp {
    fn name(&self) -> &'static str {
        env!("CARGO_CRATE_NAME")
    }

    async fn init(&self, context: &mut ProjectContext) -> cot::Result<()> {
        let user = DatabaseUser::get_by_username(context.database(), "admin").await?;
        if user.is_none() {
            DatabaseUser::create_user(context.database(), "admin", "admin").await?;
        }

        Ok(())
    }

    fn admin_model_managers(&self) -> Vec<Box<dyn AdminModelManager>> {
        vec![
            Box::new(DefaultAdminModelManager::<models::Task>::new())
        ]
    }

    fn migrations(&self) -> Vec<Box<SyncDynMigration>> {
        cot::db::migrations::wrap_migrations(migrations::MIGRATIONS)
    }

    fn router(&self) -> Router {
        Router::with_urls([
            Route::with_handler_and_name("/", index, "index"),
            
            // Auth Resources
            Route::with_handler_and_name("/api/auth/otp", api_auth::auth::send_otp, "send_otp"),
            Route::with_handler_and_name("/api/auth/verify_otp", api_auth::auth::verify_otp, "verify_otp"),
            Route::with_handler_and_name("/api/auth/me", api_auth::auth::get_current_user, "get_current_user"),
            Route::with_handler_and_name("/api/auth/logout", api_auth::auth::logout, "logout"),
            
            // Task Resources - Using AuthenticatedUser extractor for isolation
            Route::with_api_handler_and_name("/api/tasks", api_get(api_tasks::tasks::list_tasks), "list_tasks"),
            Route::with_api_handler_and_name("/api/tasks/create", api_post(api_tasks::tasks::create_task), "create_task"),
            Route::with_api_handler_and_name("/api/tasks/{id}/update", api_patch(api_tasks::tasks::update_task), "update_task"),
            Route::with_api_handler_and_name("/api/tasks/{id}/delete", api_delete(api_tasks::tasks::delete_task), "delete_task"),
        ])
    }
}

struct TaskManagerProject;

impl Project for TaskManagerProject {
    fn cli_metadata(&self) -> CliMetadata {
        cot::cli::metadata!()
    }

    fn register_apps(&self, apps: &mut AppBuilder, _context: &RegisterAppsContext) {
        apps.register_with_views(AdminApp::new(), "/admin");
        apps.register_with_views(TaskManagerApp, "");
        apps.register(DatabaseUserApp::new());
        apps.register(SessionApp::new());
        apps.register_with_views(SwaggerUi::new(), "/swagger");
    }

    fn middlewares(
        &self,
        handler: RootHandlerBuilder,
        context: &MiddlewareContext,
    ) -> RootHandler {
        handler
            .middleware(LocaleMiddleware::with_locales(vec!["pt-BR"]))
            .middleware(StaticFilesMiddleware::from_context(context))
            .middleware(AuthMiddleware::new())
            .middleware(SessionMiddleware::from_context(context)) 
            .middleware(LiveReloadMiddleware::from_context(context))
            .build()
    }
}

#[cot::main]
fn main() -> impl Project {
    env_logger::init();
    TaskManagerProject
}