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
        total_installations: i64,
        installations_by_type: IndexMap<compact_str::CompactString, i64>,
        installations_by_version: IndexMap<compact_str::CompactString, i64>,
    }

    #[utoipa::path(get, path = "/", responses(
        (status = OK, body = inline(Response)),
    ))]
    pub async fn route(state: GetState, permissions: GetPermissionManager) -> ApiResponseResult {
        permissions.has_admin_permission("settings.read")?;

        let (total_installations, installations_by_type, installations_by_version) = tokio::try_join!(
            crate::statistics::get_total_installations(&state.database),
            crate::statistics::get_installations_by_type(&state.database),
            crate::statistics::get_installations_by_version(&state.database),
        )?;

        ApiResponse::new_serialized(Response {
            total_installations,
            installations_by_type,
            installations_by_version,
        })
        .ok()
    }
}

pub fn router(state: &State) -> OpenApiRouter<State> {
    OpenApiRouter::new()
        .routes(routes!(get::route))
        .with_state(state.clone())
}
