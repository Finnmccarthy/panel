use super::State;
use utoipa_axum::{router::OpenApiRouter, routes};

mod get {
    use indexmap::IndexMap;
    use serde::Serialize;
    use shared::{
        GetState,
        models::user::GetPermissionManager,
        response::{ApiResponse, ApiResponseResult},
    };
    use utoipa::ToSchema;

    #[derive(ToSchema, Serialize)]
    struct Response {
        mcjars_type_order: IndexMap<compact_str::CompactString, Vec<compact_str::CompactString>>,
    }

    #[utoipa::path(get, path = "/", responses(
        (status = OK, body = inline(Response)),
    ))]
    pub async fn route(state: GetState, permissions: GetPermissionManager) -> ApiResponseResult {
        permissions.has_admin_permission("settings.read")?;

        let types = state
            .cache
            .cached("minecraft::mcjars::types", 60 * 30, || async {
                crate::mcjars::types(
                    crate::mcjars::ApiContext::from_settings(&state.settings).await?,
                    &state,
                )
                .await
            })
            .await?;

        let mut ordered_types = IndexMap::new();
        for (mut category, types) in types {
            category[0..1].make_ascii_uppercase();
            ordered_types.insert(category, types.into_keys().collect());
        }

        ApiResponse::new_serialized(Response {
            mcjars_type_order: ordered_types,
        })
        .ok()
    }
}

pub fn router(state: &State) -> OpenApiRouter<State> {
    OpenApiRouter::new()
        .routes(routes!(get::route))
        .with_state(state.clone())
}
