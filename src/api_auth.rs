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