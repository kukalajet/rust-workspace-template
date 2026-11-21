//! What the domain needs from the outside world, described as traits.
//!
//! The domain only depends on these traits; apps choose the real
//! implementations (adapters) and tests use fakes.

use std::{error::Error, time::SystemTime};

use crate::model::User;

/// Tells the current time.
pub trait Clock: Send + Sync {
    fn now(&self) -> SystemTime;
}

/// Stores users.
pub trait UserRepository: Send + Sync {
    /// Saves a new user.
    ///
    /// Must fail with [`RepositoryError::Conflict`] if a user with the same
    /// email already exists, atomically with the insert.
    fn insert(&self, user: &User) -> impl Future<Output = Result<(), RepositoryError>> + Send;
}

/// Why a repository operation failed.
#[derive(Debug, thiserror::Error)]
pub enum RepositoryError {
    #[error("record already exists")]
    Conflict,
    #[error("storage is unavailable")]
    Unavailable(#[source] Box<dyn Error + Send + Sync>),
}
