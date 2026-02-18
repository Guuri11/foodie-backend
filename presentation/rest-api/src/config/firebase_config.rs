pub struct FirebaseConfig {
    pub project_id: String,
}

impl FirebaseConfig {
    pub fn from_env() -> Self {
        Self {
            project_id: std::env::var("FIREBASE_PROJECT_ID")
                .expect("FIREBASE_PROJECT_ID must be set"),
        }
    }
}
