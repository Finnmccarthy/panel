use super::State;
use utoipa_axum::{router::OpenApiRouter, routes};

mod delete {
    use axum::{extract::Path, http::StatusCode};
    use serde::Serialize;
    use shared::{
        ApiError, GetState,
        models::{
            DeletableModel,
            server::{GetServer, GetServerActivityLogger},
            user::GetPermissionManager,
        },
        response::{ApiResponse, ApiResponseResult},
    };
    use utoipa::ToSchema;

    use crate::models::server_subdomain::ServerSubdomain;

    #[derive(ToSchema, Serialize)]
    struct Response {}

    #[utoipa::path(delete, path = "/", responses(
        (status = OK, body = inline(Response)),
        (status = UNAUTHORIZED, body = ApiError),
        (status = NOT_FOUND, body = ApiError),
    ), params(
        (
            "server" = uuid::Uuid,
            description = "The server ID",
            example = "123e4567-e89b-12d3-a456-426614174000",
        ),
        (
            "subdomain" = uuid::Uuid,
            description = "The subdomain ID",
            example = "123e4567-e89b-12d3-a456-426614174000",
        ),
    ))]
    pub async fn route(
        state: GetState,
        permissions: GetPermissionManager,
        server: GetServer,
        activity_logger: GetServerActivityLogger,
        Path((_server, subdomain)): Path<(String, uuid::Uuid)>,
    ) -> ApiResponseResult {
        permissions.has_server_permission("subdomains.delete")?;

        let Some(subdomain) =
            ServerSubdomain::by_server_uuid_uuid(&state.database, server.uuid, subdomain).await?
        else {
            return ApiResponse::error("subdomain not found")
                .with_status(StatusCode::NOT_FOUND)
                .ok();
        };

        subdomain.delete(&state, ()).await?;

        activity_logger
            .log(
                "server:subdomain.delete",
                serde_json::json!({
                    "uuid": subdomain.uuid,
                    "subdomain": subdomain.subdomain,
                    "domain": subdomain.domain.domain,
                }),
            )
            .await;

        ApiResponse::new_serialized(Response {}).ok()
    }
}

mod patch {
    use crate::models::server_subdomain::ServerSubdomain;
    use axum::{extract::Path, http::StatusCode};
    use garde::Validate;
    use serde::{Deserialize, Serialize};
    use shared::{
        ApiError, GetState,
        models::{
            UpdatableModel,
            server::{GetServer, GetServerActivityLogger},
            user::GetPermissionManager,
        },
        response::{ApiResponse, ApiResponseResult},
    };
    use utoipa::ToSchema;

    #[derive(ToSchema, Validate, Deserialize)]
    pub struct Payload {
        #[garde(skip)]
        allocation_uuid: uuid::Uuid,
    }

    #[derive(ToSchema, Serialize)]
    struct Response {}

    #[utoipa::path(patch, path = "/", responses(
        (status = OK, body = inline(Response)),
        (status = UNAUTHORIZED, body = ApiError),
        (status = NOT_FOUND, body = ApiError),
        (status = BAD_REQUEST, body = ApiError),
    ), params(
        (
            "server" = uuid::Uuid,
            description = "The server ID",
            example = "123e4567-e89b-12d3-a456-426614174000",
        ),
        (
            "subdomain" = uuid::Uuid,
            description = "The subdomain ID",
            example = "123e4567-e89b-12d3-a456-426614174000",
        ),
    ), request_body = inline(Payload))]
    pub async fn route(
        state: GetState,
        permissions: GetPermissionManager,
        server: GetServer,
        activity_logger: GetServerActivityLogger,
        Path((_server, subdomain)): Path<(String, uuid::Uuid)>,
        shared::Payload(data): shared::Payload<Payload>,
    ) -> ApiResponseResult {
        if let Err(errors) = shared::utils::validate_data(&data) {
            return ApiResponse::new_serialized(ApiError::new_strings_value(errors))
                .with_status(StatusCode::BAD_REQUEST)
                .ok();
        }

        permissions.has_server_permission("subdomains.update")?;

        let Some(mut subdomain) =
            ServerSubdomain::by_server_uuid_uuid(&state.database, server.uuid, subdomain).await?
        else {
            return ApiResponse::error("subdomain not found")
                .with_status(StatusCode::NOT_FOUND)
                .ok();
        };

        if let Err(err) = subdomain
            .update(
                &state,
                crate::models::server_subdomain::UpdateServerSubdomainOptions {
                    allocation_uuid: Some(data.allocation_uuid),
                },
            )
            .await
        {
            return ApiResponse::from(err).ok();
        }

        activity_logger
            .log(
                "server:subdomain.update",
                serde_json::json!({
                    "uuid": subdomain.uuid,
                    "subdomain": subdomain.subdomain,
                    "domain": subdomain.domain.domain,
                    "allocation_uuid": subdomain.allocation_uuid,
                }),
            )
            .await;

        ApiResponse::new_serialized(Response {}).ok()
    }
}

pub fn router(state: &State) -> OpenApiRouter<State> {
    OpenApiRouter::new()
        .routes(routes!(delete::route))
        .routes(routes!(patch::route))
        .with_state(state.clone())
}
