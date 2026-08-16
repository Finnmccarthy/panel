use super::State;
use utoipa_axum::{router::OpenApiRouter, routes};

mod post {
    use std::collections::HashMap;

    use axum::http::StatusCode;
    use indexmap::IndexMap;
    use serde::{Deserialize, Serialize};
    use shared::{
        GetState,
        models::{
            ByUuid, UpdatableModel,
            nest_egg::NestEgg,
            node_allocation::NodeAllocation,
            server::{GetServer, GetServerActivityLogger},
            server_variable::ServerVariable,
            user::GetPermissionManager,
        },
        response::{ApiResponse, ApiResponseResult},
    };
    use sqlx::Row;
    use utoipa::ToSchema;

    #[derive(ToSchema, Deserialize)]
    pub struct Payload {
        egg_uuid: uuid::Uuid,
        update_startup: bool,
        reinstall: bool,
        truncate_directory: bool,
    }

    #[derive(ToSchema, Serialize)]
    struct Response {}

    #[utoipa::path(post, path = "/", responses(
        (status = OK, body = inline(Response)),
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
        mut server: GetServer,
        activity_logger: GetServerActivityLogger,
        shared::Payload(data): shared::Payload<Payload>,
    ) -> ApiResponseResult {
        permissions.has_server_permission("settings.egg-changer")?;

        let settings = state.settings.get().await?;
        let extension_settings: &crate::settings::ExtensionSettingsData =
            settings.find_extension_settings()?;

        let Some(egg_group) = extension_settings.egg_groups.iter().find(|group| {
            group.eggs.contains(&data.egg_uuid) && group.affected_eggs.contains(&server.egg.uuid)
        }) else {
            return ApiResponse::error("the specified egg is not available for this server")
                .with_status(StatusCode::BAD_REQUEST)
                .ok();
        };

        let update_startup = data.update_startup || egg_group.force_update_startup;
        let reinstall = data.reinstall || egg_group.force_reinstall;
        let truncate_directory =
            data.truncate_directory || egg_group.force_reinstall_truncate_files;
        let reassign_allocations = egg_group.reassign_allocations;
        drop(settings);

        let egg = NestEgg::by_uuid_cached(&state.database, data.egg_uuid).await?;

        server
            .update(
                &state,
                shared::models::server::UpdateServerOptions {
                    egg_uuid: Some(egg.uuid),
                    startup: if update_startup {
                        egg.startup_commands
                            .get("Default")
                            .or_else(|| egg.startup_commands.values().next())
                            .cloned()
                    } else {
                        None
                    },
                    ..Default::default()
                },
            )
            .await?;

        if truncate_directory && !reinstall {
            let api_client = server
                .node
                .fetch_cached(&state.database)
                .await?
                .api_client(&state.database)
                .await?;

            let files = api_client
                .get_servers_server_files_list_directory(
                    server.uuid,
                    &wings_api::servers_server_files_list_directory::get::Query {
                        directory: Some("/".into()),
                        ..Default::default()
                    },
                )
                .await?;

            api_client
                .post_servers_server_files_delete(
                    server.uuid,
                    &wings_api::servers_server_files_delete::post::RequestBody {
                        root: "/".into(),
                        files: files.into_iter().map(|f| f.name).collect(),
                    },
                )
                .await?;
        }

        if reinstall {
            server
                .install(
                    &state,
                    truncate_directory,
                    Some(wings_api::InstallationScript {
                        container_image: egg.config_script.container,
                        entrypoint: egg.config_script.entrypoint,
                        script: egg.config_script.content.into(),
                        environment: IndexMap::new(),
                    }),
                )
                .await?;
        }

        if reassign_allocations
            && let Some(config_allocations) = server
                .egg
                .configuration(&state.database)
                .await?
                .config_allocations
        {
            let mut deployment_variables = HashMap::new();

            let node = server.node.fetch(&state.database).await?;

            let (deployment_allocation_uuid, deployment_allocation_uuids) =
                NodeAllocation::get_from_deployment(
                    &state,
                    &config_allocations.deployment,
                    &node,
                    &mut deployment_variables,
                )
                .await?;

            if !deployment_variables.is_empty() {
                let variables = ServerVariable::all_by_server_uuid_egg_uuid(
                    &state.database,
                    server.uuid,
                    server.egg.uuid,
                )
                .await?;

                for (variable_key, variable_value) in deployment_variables.into_iter() {
                    let variable_uuid = match variables
                        .iter()
                        .find(|v| v.variable.env_variable == variable_key)
                    {
                        Some(variable) => variable.variable.uuid,
                        None => continue,
                    };

                    ServerVariable::create(
                        &state.database,
                        server.uuid,
                        variable_uuid,
                        &variable_value,
                    )
                    .await?;
                }
            }

            let mut transaction = state.database.write().begin().await?;

            sqlx::query(
                "DELETE FROM server_allocations
                WHERE server_allocations.server_uuid = $1",
            )
            .bind(server.uuid)
            .execute(&mut *transaction)
            .await?;

            let allocation_uuid = if let Some(allocation_uuid) = deployment_allocation_uuid {
                Some(
                    sqlx::query(
                        "INSERT INTO server_allocations (server_uuid, allocation_uuid)
                        VALUES ($1, $2)
                        ON CONFLICT DO NOTHING
                        RETURNING uuid",
                    )
                    .bind(server.uuid)
                    .bind(allocation_uuid)
                    .fetch_one(&mut *transaction)
                    .await?
                    .try_get::<uuid::Uuid, _>(0)?,
                )
            } else {
                None
            };

            sqlx::query(
                "UPDATE servers
                SET allocation_uuid = $2
                WHERE servers.uuid = $1",
            )
            .bind(server.uuid)
            .bind(allocation_uuid)
            .execute(&mut *transaction)
            .await?;

            if !deployment_allocation_uuids.is_empty() {
                sqlx::query(
                    "INSERT INTO server_allocations (server_uuid, allocation_uuid)
                    SELECT $1, UNNEST($2::uuid[])
                    ON CONFLICT DO NOTHING",
                )
                .bind(server.uuid)
                .bind(&deployment_allocation_uuids)
                .execute(&mut *transaction)
                .await?;
            }

            transaction.commit().await?;
        }

        activity_logger
            .log(
                "server:settings.change-egg",
                serde_json::json!({
                    "egg": egg.name,
                    "update_startup": update_startup,
                    "reinstall": reinstall,
                    "truncate_directory": truncate_directory,
                }),
            )
            .await;

        ApiResponse::new_serialized(Response {}).ok()
    }
}

pub fn router(state: &State) -> OpenApiRouter<State> {
    OpenApiRouter::new()
        .routes(routes!(post::route))
        .with_state(state.clone())
}
