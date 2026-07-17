pub mod audit;
pub mod rbac;
pub mod tls;

#[derive(Clone)]
pub struct SecurityContext {
    pub user: String,
    pub roles: Vec<String>,
    pub session_id: String,
    pub csrf_token: String,
}

impl SecurityContext {
    pub fn new(user: &str) -> Self {
        Self {
            user: user.into(),
            roles: vec!["operator".into()],
            session_id: uuid::Uuid::new_v4().to_string(),
            csrf_token: uuid::Uuid::new_v4().to_string(),
        }
    }
}
