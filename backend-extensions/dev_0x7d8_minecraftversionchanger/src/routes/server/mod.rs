pub use shared::State;
use utoipa_axum::router::OpenApiRouter;

mod install;
mod installed;
mod types;

pub fn router(state: &State) -> OpenApiRouter<State> {
    OpenApiRouter::new()
        .nest("/install", install::router(state))
        .nest("/installed", installed::router(state))
        .nest("/types", types::router(state))
        .with_state(state.clone())
}
