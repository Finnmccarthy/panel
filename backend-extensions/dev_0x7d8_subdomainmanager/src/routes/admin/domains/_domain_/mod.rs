use super::State;
use utoipa_axum::{router::OpenApiRouter, routes};

mod subdomains;
mod test;

mod patch {
    use axum::{extract::Path, http::StatusCode};
    use serde::Serialize;
    use shared::{
        ApiError, GetState,
        models::{
            ByUuid, UpdatableModel, admin_activity::GetAdminActivityLogger,
            user::GetPermissionManager,
        },
        response::{ApiResponse, ApiResponseResult},
    };
    use utoipa::ToSchema;

    use crate::models::domain::Domain;

    #[derive(ToSchema, Serialize)]
    struct Response {}

    #[utoipa::path(patch, path = "/", responses(
        (status = OK, body = inline(Response)),
        (status = UNAUTHORIZED, body = ApiError),
        (status = CONFLICT, body = ApiError),
        (status = NOT_FOUND, body = ApiError),
        (status = BAD_REQUEST, body = ApiError),
    ), params(
        (
            "domain" = uuid::Uuid,
            description = "The domain ID",
            example = "123e4567-e89b-12d3-a456-426614174000",
        ),
    ), request_body = inline(crate::models::domain::UpdateDomainOptions))]
    pub async fn route(
        state: GetState,
        permissions: GetPermissionManager,
        activity_logger: GetAdminActivityLogger,
        Path(domain): Path<uuid::Uuid>,
        shared::Payload(data): shared::Payload<crate::models::domain::UpdateDomainOptions>,
    ) -> ApiResponseResult {
        if let Err(errors) = shared::utils::validate_data(&data) {
            return ApiResponse::new_serialized(ApiError::new_strings_value(errors))
                .with_status(StatusCode::BAD_REQUEST)
                .ok();
        }

        permissions.has_admin_permission("extensions.configure")?;

        let Some(mut domain) = Domain::by_uuid_optional_cached(&state.database, domain).await?
        else {
            return ApiResponse::error("domain not found")
                .with_status(StatusCode::NOT_FOUND)
                .ok();
        };

        match domain.update(&state, data).await {
            Ok(_) => (),
            Err(err) if err.is_unique_violation() => {
                return ApiResponse::error("domain with information already exists")
                    .with_status(StatusCode::CONFLICT)
                    .ok();
            }
            Err(err) => return ApiResponse::from(err).ok(),
        };

        domain.censor_provider_config(state.client.clone());

        activity_logger
            .log(
                "domain:update",
                serde_json::json!({
                    "uuid": domain.uuid,
                    "domain": domain.domain,
                    "provider": domain.provider,
                    "provider_config": domain.provider_config,
                    "eggs": domain.eggs,
                    "disallowed_subdomains_regexes": domain.disallowed_subdomains_regexes
                }),
            )
            .await;

        ApiResponse::new_serialized(Response {}).ok()
    }
}

mod delete {
    use axum::{extract::Path, http::StatusCode};
    use serde::Serialize;
    use shared::{
        ApiError, GetState,
        models::{admin_activity::GetAdminActivityLogger, user::GetPermissionManager},
        response::{ApiResponse, ApiResponseResult},
    };
    use sqlx::Row;
    use utoipa::ToSchema;

    #[derive(ToSchema, Serialize)]
    struct Response {}

    #[utoipa::path(delete, path = "/", responses(
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
        activity_logger: GetAdminActivityLogger,
        Path(domain_uuid): Path<uuid::Uuid>,
    ) -> ApiResponseResult {
        permissions.has_admin_permission("extensions.configure")?;

        let result = sqlx::query(
            "DELETE FROM dev_0x7d8_subdomainmanager_domains
            WHERE dev_0x7d8_subdomainmanager_domains.uuid = $1
            RETURNING dev_0x7d8_subdomainmanager_domains.domain",
        )
        .bind(domain_uuid)
        .fetch_optional(state.database.write())
        .await?;

        let Some(deleted_domain) = result else {
            return ApiResponse::error("domain not found")
                .with_status(StatusCode::NOT_FOUND)
                .ok();
        };

        let domain: compact_str::CompactString = deleted_domain.try_get("domain")?;

        activity_logger
            .log(
                "domain:delete",
                serde_json::json!({
                    "uuid": domain_uuid,
                    "domain": domain
                }),
            )
            .await;

        ApiResponse::new_serialized(Response {}).ok()
    }
}

pub fn router(state: &State) -> OpenApiRouter<State> {
    OpenApiRouter::new()
        .routes(routes!(patch::route))
        .routes(routes!(delete::route))
        .nest("/test", test::router(state))
        .nest("/subdomains", subdomains::router(state))
        .with_state(state.clone())
}
