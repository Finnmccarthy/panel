pub use shared::State;
use utoipa_axum::router::OpenApiRouter;

mod available;
mod switch;

pub fn router(state: &State) -> OpenApiRouter<State> {
    OpenApiRouter::new()
        .nest("/available", available::router(state))
        .nest("/switch", switch::router(state))
        .with_state(state.clone())
}
