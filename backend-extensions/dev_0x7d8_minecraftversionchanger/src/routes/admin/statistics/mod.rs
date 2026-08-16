use shared::State;
use utoipa_axum::router::OpenApiRouter;

mod total;

pub fn router(state: &State) -> OpenApiRouter<State> {
    OpenApiRouter::new()
        .nest("/total", total::router(state))
        .with_state(state.clone())
}
