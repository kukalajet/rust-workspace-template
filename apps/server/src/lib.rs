//! The HTTP layer: routes, extractors and error mapping.
//!
//! Lives in a library so integration tests can build the router
//! without starting a real server.

mod error;
mod extract;
mod routes;
mod state;

pub use routes::router;
pub use state::AppState;
