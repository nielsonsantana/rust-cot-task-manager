// src/api.rs

pub mod auth {
    use cot::db::Database;
    use cot::json::Json;
    use cot::response::{Response, ResponseExt};
    use cot::Body;
    use crate::models::{SendOtpRequest, VerifyOtpRequest};
    use crate::cqrs::{send_otp_command, verify_otp_command};
    use log::error;

    pub async fn send_otp(db: Database, req: Json<SendOtpRequest>) -> cot::Result<Response> {
        match send_otp_command(&db, &req.0.email).await {
            Ok(_) => Ok(Response::builder().status(200).body(Body::empty()).unwrap()),
            Err(e) => {
                error!("[OTP_SEND_FAILURE] Failed for email {}: {}", req.0.email, e);
                Ok(Response::builder().status(500).body(Body::empty()).unwrap())
            },
        }
    }

    pub async fn verify_otp(db: Database, req: Json<VerifyOtpRequest>) -> cot::Result<Response> {
        match verify_otp_command(&db, &req.0.email, &req.0.code).await {
            Ok(user) => {
                let json = serde_json::to_string(&user).unwrap();
                Ok(Response::builder()
                    .status(200)
                    .header("Content-Type", "application/json")
                    .body(Body::from(json))
                    .unwrap())
            },
            Err(e) => {
                if e == "Invalid OTP" {
                    Ok(Response::builder().status(401).body(Body::empty()).unwrap())
                } else {
                    error!("[OTP_VERIFY_FAILURE] Internal error for {}: {}", req.0.email, e);
                    Ok(Response::builder().status(500).body(Body::empty()).unwrap())
                }
            },
        }
    }
}

pub mod tasks {
    use cot::db::Database;
    use cot::json::Json;
    use cot::request::extractors::Path;
    use cot::response::{Response, ResponseExt};
    use cot::Body;
    use crate::models::{CreateTaskRequest, UpdateTaskRequest};
    use crate::cqrs::{list_user_tasks_query, create_task_command, update_task_command, delete_task_command};
    use log::error;

    const MOCK_USER_ID: &str = "user_123"; 

    pub async fn list_tasks(db: Database) -> cot::Result<Response> {
        match list_user_tasks_query(&db, MOCK_USER_ID).await {
            Ok(tasks) => {
                let json = serde_json::to_string(&tasks).unwrap();
                Ok(Response::builder()
                    .status(200)
                    .header("Content-Type", "application/json")
                    .body(Body::from(json))
                    .unwrap())
            },
            Err(e) => {
                error!("[TASK_LIST_FAILURE] Failed for user {}: {}", MOCK_USER_ID, e);
                Ok(Response::builder().status(500).body(Body::empty()).unwrap())
            },
        }
    }

    pub async fn create_task(db: Database, req: Json<CreateTaskRequest>) -> cot::Result<Response> {
        match create_task_command(&db, MOCK_USER_ID, &req.0.title).await {
            Ok(_) => Ok(Response::builder().status(201).body(Body::empty()).unwrap()),
            Err(e) => {
                error!("[TASK_CREATE_FAILURE] Failed for user {}: {}", MOCK_USER_ID, e);
                Ok(Response::builder().status(500).body(Body::empty()).unwrap())
            },
        }
    }

    pub async fn update_task(db: Database, Path(id): Path<String>, req: Json<UpdateTaskRequest>) -> cot::Result<Response> {
        if let Some(status) = &req.0.status {
            if let Err(e) = update_task_command(&db, &id, status).await {
                error!("[TASK_UPDATE_FAILURE] Task ID {}: {}", id, e);
                return Ok(Response::builder().status(500).body(Body::empty()).unwrap());
            }
        }
        Ok(Response::builder().status(200).body(Body::empty()).unwrap())
    }

    pub async fn delete_task(db: Database, Path(id): Path<String>) -> cot::Result<Response> {
        if let Err(e) = delete_task_command(&db, &id).await {
            error!("[TASK_DELETE_FAILURE] Task ID {}: {}", id, e);
            return Ok(Response::builder().status(500).body(Body::empty()).unwrap());
        }
        Ok(Response::builder().status(204).body(Body::empty()).unwrap())
    }
}