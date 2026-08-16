use shared::State;
use utoipa_axum::router::OpenApiRouter;

mod _2038;
mod settings;

pub fn router(state: &State) -> OpenApiRouter<State> {
    OpenApiRouter::new()
        .nest("/2038", _2038::router(state))
        .nest("/settings", settings::router(state))
        .with_state(state.clone())
}
