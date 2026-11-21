//! Domain types. Each one can only be built through validation.

mod email;
mod user;

pub use email::{Email, EmailError};
pub use user::User;
