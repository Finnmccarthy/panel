use super::State;
use utoipa_axum::{router::OpenApiRouter, routes};

mod _domain_;

mod get {
    use axum::{extract::Query, http::StatusCode};
    use serde::Serialize;
    use shared::{
        ApiError, GetState,
        models::{
            IntoAdminApiObject, Pagination, PaginationParamsWithSearch, user::GetPermissionManager,
        },
        response::{ApiResponse, ApiResponseResult},
    };
    use utoipa::ToSchema;

    #[derive(ToSchema, Serialize)]
    struct Response {
        #[schema(inline)]
        domains: Pagination<crate::models::domain::AdminApiDomain>,
    }

    #[utoipa::path(get, path = "/", responses(
        (status = OK, body = inline(Response)),
        (status = UNAUTHORIZED, body = ApiError),
        (status = BAD_REQUEST, body = ApiError),
    ), params(
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
    ) -> ApiResponseResult {
        if let Err(errors) = shared::utils::validate_data(&params) {
            return ApiResponse::new_serialized(ApiError::new_strings_value(errors))
                .with_status(StatusCode::BAD_REQUEST)
                .ok();
        }

        permissions.has_admin_permission("settings.read")?;

        let domains = crate::models::domain::Domain::all_with_pagination(
            &state.database,
            params.page,
            params.per_page,
            params.search.as_deref(),
        )
        .await?;

        ApiResponse::new_serialized(Response {
            domains: domains
                .try_async_map(|domain| domain.into_admin_api_object(&state, ()))
                .await?,
        })
        .ok()
    }
}

mod post {
    use crate::models::domain::Domain;
    use axum::http::StatusCode;
    use serde::Serialize;
    use shared::{
        ApiError, GetState,
        models::{
            CreatableModel, IntoAdminApiObject, admin_activity::GetAdminActivityLogger,
            user::GetPermissionManager,
        },
        response::{ApiResponse, ApiResponseResult},
    };
    use utoipa::ToSchema;

    #[derive(ToSchema, Serialize)]
    struct Response {
        domain: crate::models::domain::AdminApiDomain,
    }

    #[utoipa::path(post, path = "/", responses(
        (status = OK, body = inline(Response)),
        (status = UNAUTHORIZED, body = ApiError),
        (status = CONFLICT, body = ApiError),
        (status = BAD_REQUEST, body = ApiError),
    ), request_body = inline(crate::models::domain::CreateDomainOptions))]
    pub async fn route(
        state: GetState,
        permissions: GetPermissionManager,
        activity_logger: GetAdminActivityLogger,
        shared::Payload(data): shared::Payload<crate::models::domain::CreateDomainOptions>,
    ) -> ApiResponseResult {
        permissions.has_admin_permission("extensions.configure")?;

        let mut domain = match Domain::create(&state, data).await {
            Ok(domain) => domain,
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
                "domain:create",
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

        ApiResponse::new_serialized(Response {
            domain: domain.into_admin_api_object(&state, ()).await?,
        })
        .ok()
    }
}

pub fn router(state: &State) -> OpenApiRouter<State> {
    OpenApiRouter::new()
        .routes(routes!(get::route))
        .routes(routes!(post::route))
        .nest("/{domain}", _domain_::router(state))
        .with_state(state.clone())
}
