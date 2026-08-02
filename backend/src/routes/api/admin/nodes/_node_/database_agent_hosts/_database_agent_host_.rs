use super::State;
use utoipa_axum::{router::OpenApiRouter, routes};

mod delete {
    use axum::{extract::Path, http::StatusCode};
    use serde::Serialize;
    use shared::{
        ApiError, GetState,
        models::{
            DeletableModel, admin_activity::GetAdminActivityLogger, node::GetNode,
            node_database_agent_host::NodeDatabaseAgentHost, user::GetPermissionManager,
        },
        response::{ApiResponse, ApiResponseResult},
    };
    use utoipa::ToSchema;

    #[derive(ToSchema, Serialize)]
    struct Response {}

    #[utoipa::path(delete, path = "/", responses(
        (status = OK, body = inline(Response)),
        (status = NOT_FOUND, body = ApiError),
        (status = CONFLICT, body = ApiError),
    ), params(
        (
            "node" = uuid::Uuid,
            description = "The node ID",
            example = "123e4567-e89b-12d3-a456-426614174000",
        ),
        (
            "database_agent_host" = uuid::Uuid,
            description = "The database agent host ID",
            example = "123e4567-e89b-12d3-a456-426614174000",
        ),
    ))]
    pub async fn route(
        state: GetState,
        permissions: GetPermissionManager,
        node: GetNode,
        activity_logger: GetAdminActivityLogger,
        Path((_node, database_agent_host)): Path<(uuid::Uuid, uuid::Uuid)>,
    ) -> ApiResponseResult {
        permissions.has_admin_permission("nodes.database-agent-hosts")?;

        let node_database_agent_host =
            match NodeDatabaseAgentHost::by_node_uuid_database_agent_host_uuid(
                &state.database,
                node.uuid,
                database_agent_host,
            )
            .await?
            {
                Some(host) => host,
                None => {
                    return ApiResponse::error("database agent host not found")
                        .with_status(StatusCode::NOT_FOUND)
                        .ok();
                }
            };

        node_database_agent_host.delete(&state, ()).await?;

        activity_logger
            .log(
                "node:database-agent-host.delete",
                serde_json::json!({
                    "node_uuid": node.uuid,
                    "database_agent_host_uuid": node_database_agent_host.database_agent_host.uuid,
                }),
            )
            .await;

        ApiResponse::new_serialized(Response {}).ok()
    }
}

pub fn router(state: &State) -> OpenApiRouter<State> {
    OpenApiRouter::new()
        .routes(routes!(delete::route))
        .with_state(state.clone())
}
