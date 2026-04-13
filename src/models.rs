use cot::db::model;
use serde::{Deserialize, Serialize};

// --- DOMAIN MODELS (Cot ORM) --- //

#[model]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct User {
    #[model(primary_key)]
    pub id: String,
    #[model(unique)]
    pub email: String,
}

#[model]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Otp {
    #[model(primary_key)]
    pub email: String,
    pub code: String,
}

#[model]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Task {
    #[model(primary_key)]
    pub id: String,
    pub user_id: String, 
    pub title: String,
    pub status: String,
}

// --- REQUEST MODELS --- //

#[derive(Deserialize)]
pub struct SendOtpRequest {
    pub email: String,
}

#[derive(Deserialize)]
pub struct VerifyOtpRequest {
    pub email: String,
    pub code: String,
}

#[derive(Deserialize)]
pub struct CreateTaskRequest {
    pub title: String,
}

#[derive(Deserialize)]
pub struct UpdateTaskRequest {
    pub status: Option<String>,
}