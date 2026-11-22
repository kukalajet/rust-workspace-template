/// The services handlers need.
///
/// `main` fills it with real adapters; tests fill it with fakes.
#[derive(Debug)]
pub struct AppState<R, C> {
    pub user_repository: R,
    pub clock: C,
}
