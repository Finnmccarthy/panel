pub use shared::State;
use utoipa_axum::{router::OpenApiRouter, routes};

mod _subdomain_;
mod domains;

mod get {
    use crate::models::server_subdomain::ServerSubdomain;
    use axum::{extract::Query, http::StatusCode};
    use serde::Serialize;
    use shared::{
        ApiError, GetState,
        models::{
            IntoApiObject, Pagination, PaginationParamsWithSearch, server::GetServer,
            user::GetPermissionManager,
        },
        response::{ApiResponse, ApiResponseResult},
    };
    use utoipa::ToSchema;

    #[derive(ToSchema, Serialize)]
    struct Response {
        #[schema(inline)]
        subdomains: Pagination<crate::models::server_subdomain::ApiServerSubdomain>,
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
        server: GetServer,
        Query(params): Query<PaginationParamsWithSearch>,
    ) -> ApiResponseResult {
        if let Err(errors) = shared::utils::validate_data(&params) {
            return ApiResponse::new_serialized(ApiError::new_strings_value(errors))
                .with_status(StatusCode::BAD_REQUEST)
                .ok();
        }

        permissions.has_server_permission("subdomains.read")?;

        let subdomains = ServerSubdomain::all_by_server_uuid_with_pagination(
            &state.database,
            server.uuid,
            params.page,
            params.per_page,
            params.search.as_deref(),
        )
        .await?;

        ApiResponse::new_serialized(Response {
            subdomains: subdomains
                .try_async_map(|subdomain| subdomain.into_api_object(&state, &server))
                .await?,
        })
        .ok()
    }
}

mod post {
    use crate::models::{domain::Domain, server_subdomain::ServerSubdomain};
    use axum::http::StatusCode;
    use garde::Validate;
    use serde::{Deserialize, Serialize};
    use shared::{
        ApiError, GetState,
        models::{
            BaseModel, CreatableModel, IntoApiObject,
            server::{GetServer, GetServerActivityLogger},
            user::GetPermissionManager,
        },
        response::{ApiResponse, ApiResponseResult},
    };
    use utoipa::ToSchema;

    #[derive(ToSchema, Validate, Deserialize)]
    pub struct Payload {
        #[garde(skip)]
        domain_uuid: uuid::Uuid,
        #[garde(skip)]
        allocation_uuid: uuid::Uuid,

        #[garde(length(chars, min = 3, max = 32), pattern("^[a-zA-Z0-9]+$"))]
        #[schema(min_length = 3, max_length = 32, pattern = "^[a-zA-Z0-9]+$")]
        subdomain: compact_str::CompactString,
    }

    #[derive(ToSchema, Serialize)]
    struct Response {
        subdomain: crate::models::server_subdomain::ApiServerSubdomain,
    }

    #[utoipa::path(post, path = "/", responses(
        (status = OK, body = inline(Response)),
        (status = UNAUTHORIZED, body = ApiError),
        (status = BAD_REQUEST, body = ApiError),
        (status = NOT_FOUND, body = ApiError),
        (status = CONFLICT, body = ApiError),
    ), params(
        (
            "server" = uuid::Uuid,
            description = "The server ID",
            example = "123e4567-e89b-12d3-a456-426614174000",
        ),
    ), request_body = inline(Payload))]
    pub async fn route(
        state: GetState,
        permissions: GetPermissionManager,
        server: GetServer,
        activity_logger: GetServerActivityLogger,
        shared::Payload(data): shared::Payload<Payload>,
    ) -> ApiResponseResult {
        if let Err(errors) = shared::utils::validate_data(&data) {
            return ApiResponse::new_serialized(ApiError::new_strings_value(errors))
                .with_status(StatusCode::BAD_REQUEST)
                .ok();
        }

        permissions.has_server_permission("subdomains.create")?;

        let Some(domain) =
            Domain::by_uuid_egg_uuid(&state.database, data.domain_uuid, server.egg.uuid).await?
        else {
            return ApiResponse::error("domain not found or not available for this egg")
                .with_status(StatusCode::NOT_FOUND)
                .ok();
        };

        let subdomains_lock = state
            .cache
            .lock(
                format!("servers::{}::subdomains", server.uuid),
                Some(30),
                Some(5),
            )
            .await?;

        let extension_data = server.parse_model_extension::<crate::model::ServerExtension>()?;
        let limit = extension_data.subdomain_limit;

        let subdomains =
            ServerSubdomain::count_by_server_uuid(&state.database, server.uuid).await?;
        if subdomains >= limit as i64 {
            return ApiResponse::error("maximum number of subdomains reached")
                .with_status(StatusCode::EXPECTATION_FAILED)
                .ok();
        }

        for regex_str in &domain.disallowed_subdomains_regexes {
            if regex_str.is_empty() {
                continue;
            }
            if let Ok(re) = regex::Regex::new(regex_str)
                && re.is_match(&data.subdomain)
            {
                return ApiResponse::error("subdomain not allowed")
                    .with_status(StatusCode::BAD_REQUEST)
                    .ok();
            }
        }

        let mut transaction = state.database.write().begin().await?;

        let subdomain = match ServerSubdomain::create_with_transaction(
            &state,
            crate::models::server_subdomain::CreateServerSubdomainOptions {
                server: &server,
                domain: &domain,
                allocation_uuid: data.allocation_uuid,
                subdomain: data.subdomain,
            },
            &mut transaction,
        )
        .await
        {
            Ok(subdomain) => subdomain,
            Err(err) if err.is_unique_violation() => {
                return ApiResponse::error("subdomain already exists for this domain")
                    .with_status(StatusCode::CONFLICT)
                    .ok();
            }
            Err(err) => {
                transaction.rollback().await?;
                return ApiResponse::from(err).ok();
            }
        };

        transaction.commit().await?;

        drop(subdomains_lock);

        activity_logger
            .log(
                "server:subdomain.create",
                serde_json::json!({
                    "uuid": subdomain.uuid,
                    "subdomain": subdomain.subdomain,
                    "domain": subdomain.domain.domain,
                }),
            )
            .await;

        ApiResponse::new_serialized(Response {
            subdomain: subdomain.into_api_object(&state, &server).await?,
        })
        .ok()
    }
}

pub fn router(state: &State) -> OpenApiRouter<State> {
    OpenApiRouter::new()
        .routes(routes!(get::route))
        .routes(routes!(post::route))
        .nest("/{subdomain}", _subdomain_::router(state))
        .nest("/domains", domains::router(state))
        .with_state(state.clone())
}
