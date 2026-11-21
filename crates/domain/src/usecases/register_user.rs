use crate::{
    model::{Email, User},
    services::{Clock, RepositoryError, UserRepository},
};

/// Why a user could not be registered.
#[derive(Debug, thiserror::Error)]
pub enum RegisterUserError {
    #[error("a user with email {0} is already registered")]
    AlreadyRegistered(Email),
    #[error(transparent)]
    Repository(RepositoryError),
}

/// Registers a new user with the given email.
///
/// # Errors
///
/// Returns [`RegisterUserError::AlreadyRegistered`] if the email is taken,
/// or [`RegisterUserError::Repository`] if storage fails.
pub async fn register_user(
    repository: &impl UserRepository,
    clock: &impl Clock,
    email: Email,
) -> Result<User, RegisterUserError> {
    let user = User {
        email,
        registered_at: clock.now(),
    };

    match repository.insert(&user).await {
        Ok(()) => Ok(user),
        Err(RepositoryError::Conflict) => Err(RegisterUserError::AlreadyRegistered(user.email)),
        Err(err) => Err(RegisterUserError::Repository(err)),
    }
}

#[cfg(test)]
mod tests {
    use std::{
        future,
        sync::Mutex,
        time::{Duration, SystemTime},
    };

    use super::*;

    /// A clock frozen at a fixed moment, so tests are deterministic.
    struct FixedClock(SystemTime);

    impl Clock for FixedClock {
        fn now(&self) -> SystemTime {
            self.0
        }
    }

    /// Keeps users in a vector.
    #[derive(Default)]
    struct FakeRepository {
        users: Mutex<Vec<User>>,
    }

    impl UserRepository for FakeRepository {
        fn insert(&self, user: &User) -> impl Future<Output = Result<(), RepositoryError>> + Send {
            let mut users = self.users.lock().unwrap();

            let result = if users.iter().any(|existing| existing.email == user.email) {
                Err(RepositoryError::Conflict)
            } else {
                users.push(user.clone());
                Ok(())
            };

            future::ready(result)
        }
    }

    /// Fails every operation, to test error handling.
    struct BrokenRepository;

    impl UserRepository for BrokenRepository {
        fn insert(&self, _user: &User) -> impl Future<Output = Result<(), RepositoryError>> + Send {
            future::ready(Err(RepositoryError::Unavailable("disk on fire".into())))
        }
    }

    fn clock() -> FixedClock {
        FixedClock(SystemTime::UNIX_EPOCH + Duration::from_secs(1_700_000_000))
    }

    fn email(raw: &str) -> Email {
        Email::parse(raw).unwrap()
    }

    #[tokio::test]
    async fn registers_a_new_user() {
        let repo = FakeRepository::default();
        let clock = clock();

        let user = register_user(&repo, &clock, email("jane@example.com"))
            .await
            .unwrap();

        assert_eq!(user.email, email("jane@example.com"));
        assert_eq!(user.registered_at, clock.0);
        assert_eq!(*repo.users.lock().unwrap(), vec![user]);
    }

    #[tokio::test]
    async fn rejects_an_email_that_is_already_registered() {
        let repo = FakeRepository::default();
        let clock = clock();
        register_user(&repo, &clock, email("jane@example.com"))
            .await
            .unwrap();

        let result = register_user(&repo, &clock, email("jane@example.com")).await;

        assert!(matches!(
            result,
            Err(RegisterUserError::AlreadyRegistered(taken)) if taken == email("jane@example.com")
        ));
    }

    #[tokio::test]
    async fn reports_storage_failures() {
        let result = register_user(&BrokenRepository, &clock(), email("jane@example.com")).await;

        assert!(matches!(
            result,
            Err(RegisterUserError::Repository(RepositoryError::Unavailable(
                _
            )))
        ));
    }
}
