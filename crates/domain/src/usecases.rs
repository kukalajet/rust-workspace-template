//! Business operations. Each one takes the services it needs as arguments.

mod register_user;

pub use register_user::{RegisterUserError, register_user};
