pub mod tasks {
    use cot::db::Database;
    use cot::json::Json;
    use cot::request::extractors::Path;
    use cot::response::{Response, IntoResponse};
    use cot::StatusCode;
    use cot::session::Session;
    use crate::models::{CreateTaskRequest, UpdateTaskRequest};
    use crate::cqrs::{list_user_tasks_query, create_task_command, update_task_command, delete_task_command};
    use crate::auth_extractor::AuthenticatedUser;
    use log::error;

    pub async fn list_tasks(db: Database, session: Session) -> cot::Result<Response> {
        let auth_user = match AuthenticatedUser::from_session(&session).await {
            Some(u) => u,
            None => return StatusCode::UNAUTHORIZED.into_response(),
        };

        match list_user_tasks_query(&db, &auth_user.user_id).await {
            Ok(tasks) => Json(tasks).into_response(),
            Err(e) => {
                error!("[TASK_LIST_FAILURE] Failed for user {}: {}", auth_user.user_id, e);
                StatusCode::INTERNAL_SERVER_ERROR.into_response()
            },
        }
    }

    pub async fn create_task(db: Database, session: Session, req: Json<CreateTaskRequest>) -> cot::Result<Response> {
        let auth_user = match AuthenticatedUser::from_session(&session).await {
            Some(u) => u,
            None => return StatusCode::UNAUTHORIZED.into_response(),
        };

        match create_task_command(&db, &auth_user.user_id, &req.0.title).await {
            Ok(task) => Json(task).with_status(StatusCode::CREATED).into_response(),
            Err(e) => {
                error!("[TASK_CREATE_FAILURE] Failed for user {}: {}", auth_user.user_id, e);
                StatusCode::INTERNAL_SERVER_ERROR.into_response()
            },
        }
    }

    pub async fn update_task(db: Database, session: Session, Path(id): Path<String>, req: Json<UpdateTaskRequest>) -> cot::Result<Response> {
        let auth_user = match AuthenticatedUser::from_session(&session).await {
            Some(u) => u,
            None => return StatusCode::UNAUTHORIZED.into_response(),
        };

        if let Some(status) = &req.0.status {
            if let Err(e) = update_task_command(&db, &id, &auth_user.user_id, status).await {
                error!("[TASK_UPDATE_FAILURE] Task ID {} for user {}: {}", id, auth_user.user_id, e);
                return StatusCode::INTERNAL_SERVER_ERROR.into_response();
            }
        }
        StatusCode::OK.into_response()
    }

    pub async fn delete_task(db: Database, session: Session, Path(id): Path<String>) -> cot::Result<Response> {
        let auth_user = match AuthenticatedUser::from_session(&session).await {
            Some(u) => u,
            None => return StatusCode::UNAUTHORIZED.into_response(),
        };

        match delete_task_command(&db, &id, &auth_user.user_id).await {
            Ok(_) => StatusCode::NO_CONTENT.into_response(),
            Err(e) => {
                error!("[TASK_DELETE_FAILURE] Task ID {} for user {}: {}", id, auth_user.user_id, e);
                StatusCode::INTERNAL_SERVER_ERROR.into_response()
            }
        }
    }
}