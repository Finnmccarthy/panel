use super::State;
use utoipa_axum::{router::OpenApiRouter, routes};

mod get {
    use compact_str::ToCompactString;
    use serde::Serialize;
    use shared::{
        GetState,
        models::user::GetPermissionManager,
        response::{ApiResponse, ApiResponseResult},
    };
    use utoipa::ToSchema;

    #[derive(ToSchema, Serialize)]
    struct Response {
        #[schema(inline)]
        extension: crate::_2038::ExtensionInfo,
        version: compact_str::CompactString,
    }

    #[utoipa::path(get, path = "/", responses(
        (status = OK, body = inline(Response)),
    ))]
    pub async fn route(state: GetState, permissions: GetPermissionManager) -> ApiResponseResult {
        permissions.has_admin_permission("settings.read")?;

        let extension_info = crate::_2038::get_extension_info(&state).await?;

        ApiResponse::new_serialized(Response {
            extension: extension_info,
            version: state
                .extensions
                .extensions()
                .await
                .iter()
                .find(|e| e.package_name == "dev.0x7d8.minecraftversionchanger")
                .map_or("0.0.0".into(), |e| e.version.to_compact_string()),
        })
        .ok()
    }
}

pub fn router(state: &State) -> OpenApiRouter<State> {
    OpenApiRouter::new()
        .routes(routes!(get::route))
        .with_state(state.clone())
}
