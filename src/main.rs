// src/main.rs

mod api;
mod cqrs;
mod models;
mod migrations;

use cot::auth::db::DatabaseUserApp;
use cot::cli::CliMetadata;
use cot::db::migrations::SyncDynMigration;
use cot::middleware::{AuthMiddleware, LiveReloadMiddleware, SessionMiddleware};
use cot::project::{MiddlewareContext, RegisterAppsContext, RootHandler, RootHandlerBuilder};
use cot::router::{Route, Router};
use cot::static_files::StaticFilesMiddleware;
use cot::session::db::SessionApp;
use cot::{App, AppBuilder, Project};
use cot::response::{Response, ResponseExt};
use cot::Body;

async fn index() -> cot::Result<Response> {
    Ok(Response::builder()
        .status(200)
        .header("Content-Type", "text/html")
        .body(Body::from(include_str!("../templates/index.html")))
        .unwrap())
}

struct TaskManagerApp;

impl App for TaskManagerApp {
    fn name(&self) -> &'static str {
        env!("CARGO_CRATE_NAME")
    }

    fn migrations(&self) -> Vec<Box<SyncDynMigration>> {
        crate::migrations::get_migrations()
    }

    fn router(&self) -> Router {
        Router::with_urls([
            Route::with_handler_and_name("/", index, "index"),
            Route::with_handler_and_name("/api/auth/otp/send", api::auth::send_otp, "send_otp"),
            Route::with_handler_and_name("/api/auth/otp/verify", api::auth::verify_otp, "verify_otp"),
            Route::with_handler_and_name("/api/tasks", api::tasks::list_tasks, "list_tasks"),
            Route::with_handler_and_name("/api/tasks/create", api::tasks::create_task, "create_task"),
            Route::with_handler_and_name("/api/tasks/{id}/update", api::tasks::update_task, "update_task"),
            Route::with_handler_and_name("/api/tasks/{id}/delete", api::tasks::delete_task, "delete_task"),
        ])
    }
}

struct TaskManagerProject;

impl Project for TaskManagerProject {
    fn cli_metadata(&self) -> CliMetadata {
        cot::cli::metadata!()
    }

    fn register_apps(&self, apps: &mut AppBuilder, _context: &RegisterAppsContext) {
        apps.register_with_views(TaskManagerApp, "");
        apps.register(DatabaseUserApp::new());
        apps.register(SessionApp::new());
    }

    fn middlewares(
        &self,
        handler: RootHandlerBuilder,
        context: &MiddlewareContext,
    ) -> RootHandler {
        handler
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