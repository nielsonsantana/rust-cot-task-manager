use crate::models::{Otp, Task, User};
use cot::db::{Database, Model, query};
use uuid::Uuid;
use chrono::{Duration, Utc};

// --- COMMANDS --- //

pub async fn send_otp_command(db: &Database, email: &str) -> Result<(), String> {
    let mock_code = "123456";
    let expires_at = Utc::now() + Duration::seconds(60 * 3);
    let otp_record = query!(Otp, $email == email).get(db).await.map_err(|e| e.to_string())?; 
    
    if let Some(mut record) = otp_record {
        record.code = mock_code.to_string();
        record.expires_at = expires_at;
        record.save(db).await.map_err(|e| e.to_string())?;
    } else {
        let mut new_otp = Otp {
            email: email.to_string(),
            code: mock_code.to_string(),
            expires_at,
        };
        new_otp.save(db).await.map_err(|e| e.to_string())?;
    }

    Ok(())
}

pub async fn verify_otp_command(db: &Database, email: &str, code: &str) -> Result<User, String> {
    let otp_record = query!(Otp, $email == email).get(db).await.map_err(|e| e.to_string())?; 

    if let Some(record) = otp_record {
        if record.expires_at < Utc::now() {
            query!(Otp, $email == email).delete(db).await.map_err(|e| e.to_string())?;
            return Err("OTP expired".to_string());
        }

        if record.code == code {
            query!(Otp, $email == email).delete(db).await.map_err(|e| e.to_string())?;
            
            let user = query!(User, $email == email).get(db).await.map_err(|e| e.to_string())?;

            if let Some(existing_user) = user {
                return Ok(existing_user);
            } else {
                let mut new_user = User {
                    id: Uuid::new_v4().to_string(),
                    email: email.to_string(),
                };
                new_user.save(db).await.map_err(|e| e.to_string())?;
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

pub async fn update_task_command(db: &Database, task_id: &str, status: &str) -> Result<(), String> {
    let task_opt = query!(Task, $id == task_id).get(db).await.map_err(|e| e.to_string())?;

    if let Some(mut task) = task_opt {
        task.status = status.to_string();
        task.save(db).await.map_err(|e| e.to_string())?;
    }
    
    Ok(())
}

pub async fn delete_task_command(db: &Database, task_id: &str) -> Result<(), String> {
    query!(Task, $id == task_id).delete(db).await.map_err(|e| e.to_string())?;
    Ok(())
}

// --- QUERIES --- //

pub async fn list_user_tasks_query(db: &Database, user_id: &str) -> Result<Vec<Task>, String> {
    query!(Task, $user_id == user_id)
        .all(db)
        .await
        .map_err(|e| e.to_string())
}