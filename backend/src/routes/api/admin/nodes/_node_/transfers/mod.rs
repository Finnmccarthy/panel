use super::State;
use utoipa_axum::{router::OpenApiRouter, routes};

mod servers;
mod ws;

mod get {
    use indexmap::IndexMap;
    use serde::Serialize;
    use shared::{
        ApiError, GetState,
        models::{node::GetNode, user::GetPermissionManager},
        response::{ApiResponse, ApiResponseResult},
    };
    use utoipa::ToSchema;

    #[derive(ToSchema, Serialize)]
    struct Response {
        transfers: IndexMap<uuid::Uuid, wings_api::TransferProgress>,
    }

    #[utoipa::path(get, path = "/", responses(
        (status = OK, body = inline(Response)),
        (status = UNAUTHORIZED, body = ApiError),
    ), params(
        (
            "node" = uuid::Uuid,
            description = "The node ID",
            example = "123e4567-e89b-12d3-a456-426614174000",
        ),
    ))]
    pub async fn route(
        state: GetState,
        permissions: GetPermissionManager,
        node: GetNode,
    ) -> ApiResponseResult {
        permissions.has_admin_permission("nodes.transfers")?;

        let transfers = node
            .api_client(&state.database)
            .await?
            .get_transfers()
            .await?;

        ApiResponse::new_serialized(Response { transfers }).ok()
    }
}

pub fn router(state: &State) -> OpenApiRouter<State> {
    OpenApiRouter::new()
        .routes(routes!(get::route))
        .nest("/ws", ws::router(state))
        .nest("/servers", servers::router(state))
        .with_state(state.clone())
}
