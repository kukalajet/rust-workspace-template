use std::time::SystemTime;

use super::Email;

/// A registered user.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct User {
    pub email: Email,
    pub registered_at: SystemTime,
}
