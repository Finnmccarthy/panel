pub use shared::State;
use utoipa_axum::{router::OpenApiRouter, routes};

mod get {
    use crate::models::domain::Domain;
    use serde::Serialize;
    use shared::{
        ApiError, GetState,
        models::{IntoApiObject, server::GetServer, user::GetPermissionManager},
        prelude::AsyncIteratorExt,
        response::{ApiResponse, ApiResponseResult},
    };
    use utoipa::ToSchema;

    #[derive(ToSchema, Serialize)]
    struct Response {
        domains: Vec<crate::models::domain::ApiDomain>,
    }

    #[utoipa::path(get, path = "/", responses(
        (status = OK, body = inline(Response)),
        (status = UNAUTHORIZED, body = ApiError),
    ), params(
        (
            "server" = uuid::Uuid,
            description = "The server ID",
            example = "123e4567-e89b-12d3-a456-426614174000",
        ),
    ))]
    pub async fn route(
        state: GetState,
        permissions: GetPermissionManager,
        server: GetServer,
    ) -> ApiResponseResult {
        permissions.has_server_permission("subdomains.read")?;

        let domains = state
            .cache
            .cached(&format!("domains::{}", server.egg.uuid), 30, || async {
                Domain::all_by_egg_uuid(&state.database, server.egg.uuid).await
            })
            .await?;

        ApiResponse::new_serialized(Response {
            domains: domains
                .into_iter()
                .map(|domain| domain.into_api_object(&state, ()))
                .try_collect_async_vec()
                .await?,
        })
        .ok()
    }
}

pub fn router(state: &State) -> OpenApiRouter<State> {
    OpenApiRouter::new()
        .routes(routes!(get::route))
        .with_state(state.clone())
}
