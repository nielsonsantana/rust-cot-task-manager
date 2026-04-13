pub mod auth {
    use cot::db::Database;
    use cot::json::Json;
    use cot::response::{Response, IntoResponse};
    use cot::StatusCode;
    use crate::models::{SendOtpRequest, VerifyOtpRequest};
    use crate::cqrs::{send_otp_command, verify_otp_command};
    use log::error;

    pub async fn send_otp(db: Database, req: Json<SendOtpRequest>) -> cot::Result<Response> {
        match send_otp_command(&db, &req.0.email).await {
            Ok(_) => StatusCode::OK.into_response(),
            Err(e) => {
                error!("[OTP_SEND_FAILURE] Failed for email {}: {}", req.0.email, e);
                StatusCode::INTERNAL_SERVER_ERROR.into_response()
            },
        }
    }

    pub async fn verify_otp(db: Database, req: Json<VerifyOtpRequest>) -> cot::Result<Response> {
        match verify_otp_command(&db, &req.0.email, &req.0.code).await {
            Ok(user) => Json(user).with_status(StatusCode::CREATED).into_response(),
            Err(e) => {
                if e == "Invalid OTP" {
                    StatusCode::UNAUTHORIZED.into_response()
                } else {
                    error!("[OTP_VERIFY_FAILURE] Internal error for {}: {}", req.0.email, e);
                    StatusCode::INTERNAL_SERVER_ERROR.into_response()
                }
            },
        }
    }
}

pub mod tasks {
    use cot::db::Database;
    use cot::json::Json;
    use cot::request::extractors::Path;
    use cot::response::{Response, IntoResponse};
    use cot::StatusCode;
    use crate::models::{CreateTaskRequest, UpdateTaskRequest};
    use crate::cqrs::{list_user_tasks_query, create_task_command, update_task_command, delete_task_command};
    use log::error;

    const MOCK_USER_ID: &str = "user_123"; 

    pub async fn list_tasks(db: Database) -> cot::Result<Response> {
        match list_user_tasks_query(&db, MOCK_USER_ID).await {
            Ok(tasks) => Json(tasks).into_response(),
            Err(e) => {
                error!("[TASK_LIST_FAILURE] Failed for user {}: {}", MOCK_USER_ID, e);
                StatusCode::INTERNAL_SERVER_ERROR.into_response()
            },
        }
    }

    pub async fn create_task(db: Database, req: Json<CreateTaskRequest>) -> cot::Result<Response> {
        match create_task_command(&db, MOCK_USER_ID, &req.0.title).await {
            Ok(task) => Json(task).with_status(StatusCode::CREATED).into_response(),
            Err(e) => {
                error!("[TASK_CREATE_FAILURE] Failed for user {}: {}", MOCK_USER_ID, e);
                StatusCode::INTERNAL_SERVER_ERROR.into_response()
            },
        }
    }

    pub async fn update_task(db: Database, Path(id): Path<String>, req: Json<UpdateTaskRequest>) -> cot::Result<Response> {
        if let Some(status) = &req.0.status {
            if let Err(e) = update_task_command(&db, &id, status).await {
                error!("[TASK_UPDATE_FAILURE] Task ID {}: {}", id, e);
                return StatusCode::INTERNAL_SERVER_ERROR.into_response();
            }
        }
        StatusCode::OK.into_response()
    }

    pub async fn delete_task(db: Database, Path(id): Path<String>) -> cot::Result<Response> {
        match delete_task_command(&db, &id).await {
            Ok(_) => StatusCode::NO_CONTENT.into_response(),
            Err(e) => {
                error!("[TASK_DELETE_FAILURE] Task ID {}: {}", id, e);
                StatusCode::INTERNAL_SERVER_ERROR.into_response()
            }
        }
    }
}