pub mod auth {
    use cot::db::Database;
    use cot::json::Json;
    use cot::response::{Response, IntoResponse};
    use cot::StatusCode;
    use cot::session::Session;
    use crate::models::{SendOtpRequest, VerifyOtpRequest, UserResponse, SessionUserData};
    use crate::cqrs::{send_otp_command, verify_otp_command, get_session_user_query};
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

    pub async fn verify_otp(db: Database, session: Session, json_req: Json<VerifyOtpRequest>) -> cot::Result<Response> {
        match verify_otp_command(&db, &json_req.0.email, &json_req.0.code).await {
            Ok(user) => {
                let _user_data = SessionUserData {
                    user_id: user.id().to_string(),
                    email: json_req.0.email.clone(),
                };
                
                if session.insert("user_id", user.id().to_string()).await.is_err() {
                    error!("[SESSION_ERROR] Failed to insert user_id");
                    return StatusCode::INTERNAL_SERVER_ERROR.into_response();
                }
                
                if session.insert("email", json_req.0.email.clone()).await.is_err() {
                    error!("[SESSION_ERROR] Failed to insert email");
                    return StatusCode::INTERNAL_SERVER_ERROR.into_response();
                }

                if session.save().await.is_err() {
                    error!("[SESSION_ERROR] Failed to save session");
                    return StatusCode::INTERNAL_SERVER_ERROR.into_response();
                }

                let res = UserResponse {
                    username: json_req.0.email.clone(),
                    user_id: user.id().to_string(),
                };
                Json(res).with_status(StatusCode::CREATED).into_response()
            },
            Err(e) => {
                if e == "Invalid OTP" {
                    StatusCode::UNAUTHORIZED.into_response()
                } else if e == "OTP expired" {
                    StatusCode::UNAUTHORIZED.with_body("OTP expired".to_string()).into_response()
                } else {
                    error!("[OTP_VERIFY_FAILURE] Internal error for {}: {}", json_req.0.email, e);
                    StatusCode::INTERNAL_SERVER_ERROR.into_response()
                }
            },
        }
    }

    pub async fn get_current_user(_db: Database, session: Session) -> cot::Result<Response> {
        match get_session_user_query(&session).await {
            Ok(Some(user_data)) => Json(user_data).into_response(),
            Ok(None) => StatusCode::UNAUTHORIZED.into_response(),
            Err(e) => {
                error!("[SESSION_ERROR] Failed to get session user: {}", e);
                StatusCode::INTERNAL_SERVER_ERROR.into_response()
            },
        }
    }
}