//! Pure business logic: types, rules and the services they need.
//! No I/O happens here (enforced by `crates/domain/clippy.toml`).

pub mod model;
pub mod services;
pub mod usecases;
