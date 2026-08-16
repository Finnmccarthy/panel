use super::State;
use utoipa_axum::{router::OpenApiRouter, routes};

mod post {
    use axum::{extract::Path, http::StatusCode};
    use serde::Serialize;
    use shared::{
        ApiError, GetState,
        models::user::GetPermissionManager,
        response::{ApiResponse, ApiResponseResult},
    };
    use sqlx::Row;
    use utoipa::ToSchema;

    #[derive(ToSchema, Serialize)]
    struct Response {}

    #[utoipa::path(post, path = "/", responses(
        (status = OK, body = inline(Response)),
        (status = UNAUTHORIZED, body = ApiError),
        (status = NOT_FOUND, body = ApiError),
    ), params(
        (
            "domain" = uuid::Uuid,
            description = "The domain ID",
            example = "123e4567-e89b-12d3-a456-426614174000",
        ),
    ))]
    pub async fn route(
        state: GetState,
        permissions: GetPermissionManager,
        Path(domain_uuid): Path<uuid::Uuid>,
    ) -> ApiResponseResult {
        permissions.has_admin_permission("extensions.configure")?;

        let Some(row) = sqlx::query(
            "SELECT dev_0x7d8_subdomainmanager_domains.provider,
            dev_0x7d8_subdomainmanager_domains.provider_config
            FROM dev_0x7d8_subdomainmanager_domains
            WHERE dev_0x7d8_subdomainmanager_domains.uuid = $1",
        )
        .bind(domain_uuid)
        .fetch_optional(state.database.read())
        .await?
        else {
            return ApiResponse::error("domain not found")
                .with_status(StatusCode::NOT_FOUND)
                .ok();
        };

        let provider: crate::providers::Provider = row.try_get("provider")?;
        let provider_config: serde_json::Value = row.try_get("provider_config")?;

        if let Err(err) = provider
            .build(state.client.clone())
            .test(&provider_config)
            .await
        {
            let (err, status) = shared::response::extract_readable_error(&err)
                .unwrap_or_else(|| (err.to_string(), StatusCode::INTERNAL_SERVER_ERROR));

            return ApiResponse::error(err).with_status(status).ok();
        }

        ApiResponse::new_serialized(Response {}).ok()
    }
}

pub fn router(state: &State) -> OpenApiRouter<State> {
    OpenApiRouter::new()
        .routes(routes!(post::route))
        .with_state(state.clone())
}
