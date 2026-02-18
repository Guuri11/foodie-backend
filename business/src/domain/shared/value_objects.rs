use serde::{Deserialize, Serialize};

/// Represents a user identifier (Firebase UID).
/// Used to isolate data between users.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct UserId(String);

impl UserId {
    /// Creates a new UserId from any type that can be converted into a String.
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    /// Returns the inner string as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for UserId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for UserId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for UserId {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_create_user_id_from_string() {
        let user_id = UserId::new("firebase-uid-123".to_string());
        assert_eq!(user_id.as_str(), "firebase-uid-123");
    }

    #[test]
    fn should_create_user_id_from_str() {
        let user_id = UserId::new("firebase-uid-456");
        assert_eq!(user_id.as_str(), "firebase-uid-456");
    }

    #[test]
    fn should_display_user_id() {
        let user_id = UserId::new("test-user");
        assert_eq!(format!("{}", user_id), "test-user");
    }

    #[test]
    fn should_compare_user_ids_for_equality() {
        let user_id_1 = UserId::new("same-user");
        let user_id_2 = UserId::new("same-user");
        let user_id_3 = UserId::new("different-user");

        assert_eq!(user_id_1, user_id_2);
        assert_ne!(user_id_1, user_id_3);
    }

    #[test]
    fn should_clone_user_id() {
        let user_id = UserId::new("clonable-user");
        let cloned = user_id.clone();
        assert_eq!(user_id, cloned);
    }

    #[test]
    fn should_convert_from_string() {
        let user_id: UserId = "from-string".to_string().into();
        assert_eq!(user_id.as_str(), "from-string");
    }

    #[test]
    fn should_convert_from_str() {
        let user_id: UserId = "from-str".into();
        assert_eq!(user_id.as_str(), "from-str");
    }
}
