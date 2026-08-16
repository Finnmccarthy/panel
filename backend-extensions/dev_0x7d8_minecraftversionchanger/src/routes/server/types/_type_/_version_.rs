use super::State;
use utoipa_axum::{router::OpenApiRouter, routes};

mod get {
    use axum::extract::{Path, Query};
    use serde::Serialize;
    use shared::{
        GetState,
        models::{Pagination, PaginationParamsWithSearch, user::GetPermissionManager},
        response::{ApiResponse, ApiResponseResult},
    };
    use utoipa::ToSchema;

    #[derive(ToSchema, Serialize)]
    struct Response {
        #[schema(inline)]
        builds: Pagination<crate::mcjars::MinecraftBuild>,
    }

    #[utoipa::path(get, path = "/", responses(
        (status = OK, body = inline(Response)),
    ), params(
        (
            "server" = uuid::Uuid,
            description = "The server ID",
            example = "123e4567-e89b-12d3-a456-426614174000",
        ),
        (
            "type" = String,
            description = "The minecraft type identifier",
            example = "VANILLA",
        ),
        (
            "version" = String,
            description = "The minecraft version identifier",
            example = "1.21.1",
        ),
        (
            "page" = i64, Query,
            description = "The page number",
            example = "1",
        ),
        (
            "per_page" = i64, Query,
            description = "The number of items per page",
            example = "10",
        ),
        (
            "search" = Option<String>, Query,
            description = "Search term for items",
        ),
    ))]
    pub async fn route(
        state: GetState,
        permissions: GetPermissionManager,
        Query(params): Query<PaginationParamsWithSearch>,
        Path((_server, r#type, version)): Path<(String, String, String)>,
    ) -> ApiResponseResult {
        permissions.has_server_permission("files.read")?;

        let builds = state
            .cache
            .cached(
                &format!(
                    "minecraft::mcjars::types::{}::versions::{}::builds:{}:{}:{}",
                    r#type,
                    version,
                    params.page,
                    params.per_page,
                    params.search.as_deref().unwrap_or("")
                ),
                60 * 15,
                || async {
                    crate::mcjars::builds(
                        crate::mcjars::ApiContext::from_settings(&state.settings).await?,
                        &state,
                        &r#type,
                        &version,
                        params.page,
                        params.per_page,
                        params.search.as_deref(),
                    )
                    .await
                },
            )
            .await?;

        ApiResponse::new_serialized(Response { builds }).ok()
    }
}

pub fn router(state: &State) -> OpenApiRouter<State> {
    OpenApiRouter::new()
        .routes(routes!(get::route))
        .with_state(state.clone())
}
