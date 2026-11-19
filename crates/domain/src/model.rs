//! Domain types. Each one can only be built through validation.

mod email;

pub use email::{Email, EmailError};
