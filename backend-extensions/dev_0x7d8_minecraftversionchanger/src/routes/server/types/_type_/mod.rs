use super::State;
use utoipa_axum::{router::OpenApiRouter, routes};

mod _version_;

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
        versions: Pagination<crate::mcjars::MinecraftVersion>,
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
        Path((_server, r#type)): Path<(String, String)>,
    ) -> ApiResponseResult {
        permissions.has_server_permission("files.read")?;

        let versions = state
            .cache
            .cached(
                &format!(
                    "minecraft::mcjars::types::{}::versions:{}:{}:{}",
                    r#type,
                    params.page,
                    params.per_page,
                    params.search.as_deref().unwrap_or("")
                ),
                60 * 15,
                || async {
                    crate::mcjars::versions(
                        crate::mcjars::ApiContext::from_settings(&state.settings).await?,
                        &state,
                        &r#type,
                        params.page,
                        params.per_page,
                        params.search.as_deref(),
                    )
                    .await
                },
            )
            .await?;

        ApiResponse::new_serialized(Response { versions }).ok()
    }
}

pub fn router(state: &State) -> OpenApiRouter<State> {
    OpenApiRouter::new()
        .routes(routes!(get::route))
        .nest("/{version}", _version_::router(state))
        .with_state(state.clone())
}
