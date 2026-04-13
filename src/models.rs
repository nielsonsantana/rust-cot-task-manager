// src/models.rs

use cot::db::model;
use serde::{Deserialize, Serialize};
use schemars::JsonSchema;

// --- DOMAIN MODELS (Cot ORM) --- //

#[model]
#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
pub struct User {
    #[model(primary_key)]
    pub id: String,
    #[model(unique)]
    pub email: String,
}

#[model]
#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
pub struct Otp {
    #[model(primary_key)]
    pub email: String,
    pub code: String,
}

#[model]
#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
pub struct Task {
    #[model(primary_key)]
    pub id: String,
    pub user_id: String, 
    pub title: String,
    pub status: String,
}

// --- REQUEST MODELS --- //

#[derive(Deserialize, JsonSchema)]
pub struct SendOtpRequest {
    pub email: String,
}

#[derive(Deserialize, JsonSchema)]
pub struct VerifyOtpRequest {
    pub email: String,
    pub code: String,
}

#[derive(Deserialize, JsonSchema)]
pub struct CreateTaskRequest {
    pub title: String,
}

#[derive(Deserialize, JsonSchema)]
pub struct UpdateTaskRequest {
    pub status: Option<String>,
}