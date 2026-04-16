use cot::session::Session;

pub struct AuthenticatedUser {
    pub user_id: String,
    #[allow(dead_code)]
    pub email: String,
}

impl AuthenticatedUser {
    pub async fn from_session(session: &Session) -> Option<Self> {
        let user_id = session.get::<String>("user_id").await.unwrap_or_default()?;
        let email = session.get::<String>("email").await.unwrap_or_default().unwrap_or_default();
        Some(AuthenticatedUser { user_id, email })
    }
}