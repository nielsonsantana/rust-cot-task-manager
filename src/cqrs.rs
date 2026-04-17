use crate::models::{Otp, Task, SessionUserData};
use crate::config;
use cot::auth::db::DatabaseUser;
use cot::db::{Database, Model, query};
use uuid::Uuid;
use chrono::{Duration, Utc};
use rand::Rng;

// --- COMMANDS --- //

pub async fn send_otp_command(db: &Database, email: &str) -> Result<(), String> {
    // Use mocked code for localhost environment, otherwise generate random
    let otp_code = if config::is_localhost() {
        "123456".to_string()
    } else {
        // Scope the ThreadRng usage so it is dropped before the .await point.
        // This prevents the returned Future from becoming !Send.
        let code = {
            let mut rng = rand::thread_rng();
            rng.gen_range(100000..=999999).to_string()
        };
        code
    };
    
    let expires_at = Utc::now() + Duration::seconds(60 * 3);
    let otp_record = query!(Otp, $email == email).get(db).await.map_err(|e| e.to_string())?; 
    
    if let Some(mut record) = otp_record {
        record.code = otp_code;
        record.expires_at = expires_at;
        record.save(db).await.map_err(|e| e.to_string())?;
    } else {
        let mut new_otp = Otp {
            email: email.to_string(),
            code: otp_code,
            expires_at,
        };
        new_otp.save(db).await.map_err(|e| e.to_string())?;
    }

    Ok(())
}

pub async fn verify_otp_command(db: &Database, email: &str, code: &str) -> Result<DatabaseUser, String> {
    let otp_record = query!(Otp, $email == email).get(db).await.map_err(|e| e.to_string())?; 

    if let Some(record) = otp_record {
        if record.expires_at < Utc::now() {
            query!(Otp, $email == email).delete(db).await.map_err(|e| e.to_string())?;
            return Err("OTP expired".to_string());
        }

        if record.code == code {
            query!(Otp, $email == email).delete(db).await.map_err(|e| e.to_string())?;
            
            let user = DatabaseUser::get_by_username(db, email).await.map_err(|e| e.to_string())?;

            if let Some(existing_user) = user {
                return Ok(existing_user);
            } else {
                let temp_pass = Uuid::new_v4().to_string();
                DatabaseUser::create_user(db, email, temp_pass).await.map_err(|e| e.to_string())?;
                let new_user = DatabaseUser::get_by_username(db, email)
                    .await
                    .map_err(|e| e.to_string())?
                    .ok_or_else(|| "Failed to retrieve created user".to_string())?;
                return Ok(new_user);
            }
        }
    }
    Err("Invalid OTP".to_string())
}

pub async fn create_task_command(db: &Database, user_id: &str, title: &str) -> Result<Task, String> {
    let mut task = Task {
        id: Uuid::new_v4().to_string(),
        user_id: user_id.to_string(),
        title: title.to_string(),
        status: "Pending".to_string(),
    };

    task.save(db).await.map_err(|e| e.to_string())?;
    Ok(task)
}

pub async fn update_task_command(db: &Database, task_id: &str, user_id: &str, status: &str) -> Result<(), String> {
    let task_opt = query!(Task, $id == task_id && $user_id == user_id).get(db).await.map_err(|e| e.to_string())?;

    if let Some(mut task) = task_opt {
        task.status = status.to_string();
        task.save(db).await.map_err(|e| e.to_string())?;
    } else {
        return Err(rust_i18n::t!("task_not_found_or_unauthorized").to_string());
    }
    
    Ok(())
}

pub async fn delete_task_command(db: &Database, task_id: &str, user_id: &str) -> Result<(), String> {
    query!(Task, $id == task_id && $user_id == user_id).delete(db).await.map_err(|e| e.to_string())?;
    Ok(())
}

// --- QUERIES --- //

pub async fn list_user_tasks_query(db: &Database, user_id: &str) -> Result<Vec<Task>, String> {
    query!(Task, $user_id == user_id)
        .all(db)
        .await
        .map_err(|e| e.to_string())
}

pub async fn get_session_user_query(session: &cot::session::Session) -> Result<Option<SessionUserData>, String> {
    let user_id = session.get::<String>("user_id").await
        .map_err(|e| e.to_string())?
        .unwrap_or_default();
    
    if user_id.is_empty() {
        return Ok(None);
    }
    
    let email = session.get::<String>("email").await
        .map_err(|e| e.to_string())?
        .unwrap_or_default();

    Ok(Some(SessionUserData {
        user_id,
        email,
    }))
}