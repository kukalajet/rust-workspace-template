//! Adapters that keep data in process memory.
//!
//! Useful before a real database exists, and in tests. Everything is lost
//! when the process exits.

use std::{
    collections::{HashMap, hash_map::Entry},
    future,
    sync::Mutex,
};

use domain::{
    model::{Email, User},
    services::{RepositoryError, UserRepository},
};

/// Stores users in a `HashMap` keyed by email.
#[derive(Debug, Default)]
pub struct InMemoryUserRepository {
    users: Mutex<HashMap<Email, User>>,
}

impl InMemoryUserRepository {
    fn try_insert(&self, user: &User) -> Result<(), RepositoryError> {
        let mut users = self
            .users
            .lock()
            .map_err(|_| RepositoryError::Unavailable("user store lock is poisoned".into()))?;

        match users.entry(user.email.clone()) {
            Entry::Occupied(_) => Err(RepositoryError::Conflict),
            Entry::Vacant(slot) => {
                slot.insert(user.clone());

                Ok(())
            }
        }
    }
}

impl UserRepository for InMemoryUserRepository {
    fn insert(&self, user: &User) -> impl Future<Output = Result<(), RepositoryError>> + Send {
        future::ready(self.try_insert(user))
    }
}

#[cfg(test)]
mod tests {
    use std::time::SystemTime;

    use super::*;

    fn user(email: &str) -> User {
        User {
            email: Email::parse(email).unwrap(),
            registered_at: SystemTime::UNIX_EPOCH,
        }
    }

    #[tokio::test]
    async fn inserts_a_new_user() {
        let repository = InMemoryUserRepository::default();

        let result = repository.insert(&user("jane@example.com")).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn rejects_a_duplicate_email() {
        let repository = InMemoryUserRepository::default();
        repository.insert(&user("jane@example.com")).await.unwrap();

        let result = repository.insert(&user("jane@example.com")).await;

        assert!(matches!(result, Err(RepositoryError::Conflict)));
    }
}
