use shared::State;
use utoipa_axum::router::OpenApiRouter;

mod _2038;
#[path = "domains/mod.rs"]
mod domains;

pub fn router(state: &State) -> OpenApiRouter<State> {
    OpenApiRouter::new()
        .nest("/2038", _2038::router(state))
        .nest("/domains", domains::router(state))
        .with_state(state.clone())
}
