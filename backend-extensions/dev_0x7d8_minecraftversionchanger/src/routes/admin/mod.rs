use shared::State;
use utoipa_axum::router::OpenApiRouter;

mod _2038;
mod default_order;
mod settings;
mod statistics;

pub fn router(state: &State) -> OpenApiRouter<State> {
    OpenApiRouter::new()
        .nest("/2038", _2038::router(state))
        .nest("/settings", settings::router(state))
        .nest("/default-order", default_order::router(state))
        .nest("/statistics", statistics::router(state))
        .with_state(state.clone())
}
