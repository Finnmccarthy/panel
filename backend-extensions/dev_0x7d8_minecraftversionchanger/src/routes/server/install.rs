use super::State;
use utoipa_axum::{router::OpenApiRouter, routes};

mod post {
    use serde::{Deserialize, Serialize};
    use shared::{
        GetState,
        models::{
            ByUuid, UpdatableModel,
            nest_egg::NestEgg,
            server::{GetServer, GetServerActivityLogger},
            user::{GetPermissionManager, GetUser},
        },
        response::{ApiResponse, ApiResponseResult},
    };
    use std::pin::Pin;
    use utoipa::ToSchema;

    #[derive(ToSchema, Deserialize)]
    pub struct Payload {
        build_uuid: uuid::Uuid,
        truncate_directory: bool,
        accept_eula: bool,
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
        user: GetUser,
        mut server: GetServer,
        activity_logger: GetServerActivityLogger,
        axum::Json(data): axum::Json<Payload>,
    ) -> ApiResponseResult {
        permissions.has_server_permission("files.update")?;

        let (build, version) = state
            .cache
            .cached(
                &format!("minecraft::mcjars::build::{}", data.build_uuid),
                60,
                || async {
                    crate::mcjars::lookup_uuid(
                        crate::mcjars::ApiContext::from_settings(&state.settings).await?,
                        &state,
                        data.build_uuid,
                    )
                    .await
                },
            )
            .await?;
        let node = server.node.fetch_cached(&state.database).await?;

        let mut to_delete_files = vec!["libraries".into()];

        let api_client = node.api_client(&state.database).await?;

        if data.truncate_directory {
            let files = api_client
                .get_servers_server_files_list_directory(
                    server.uuid,
                    &wings_api::servers_server_files_list_directory::get::Query {
                        directory: Some("/".into()),
                        ..Default::default()
                    },
                )
                .await?;

            for file in files {
                to_delete_files.push(file.name);
            }
        }

        api_client
            .post_servers_server_files_delete(
                server.uuid,
                &wings_api::servers_server_files_delete::post::RequestBody {
                    root: "/".into(),
                    files: to_delete_files,
                },
            )
            .await?;

        if data.accept_eula {
            api_client
                .post_servers_server_files_write(
                    server.uuid,
                    wings_api::client::AsyncRequestReader::new(std::io::Cursor::new(
                        "eula=true\n".as_bytes().to_vec(),
                    )),
                    &wings_api::servers_server_files_write::post::Query {
                        file: Some("eula.txt".into()),
                        user: Some(user.uuid),
                        ..Default::default()
                    },
                )
                .await?;
        }

        let settings = state.settings.get().await?;
        let extension_settings =
            settings.find_extension_settings::<crate::settings::ExtensionSettingsData>()?;

        if extension_settings.collect_installation_statistics {
            crate::statistics::insert_installation(&state.database, server.uuid, &build).await?;
        }

        if let Some(egg_assignment) = extension_settings
            .type_egg_assignments
            .iter()
            .find(|egg_assignment| egg_assignment.affected_types.contains(&build.r#type))
            && let Some(egg) =
                NestEgg::by_uuid_optional(&state.database, egg_assignment.egg).await?
        {
            server
                .update(
                    &state,
                    shared::models::server::UpdateServerOptions {
                        egg_uuid: Some(egg.uuid),
                        startup: egg
                            .startup_commands
                            .get("Default")
                            .or_else(|| egg.startup_commands.values().next())
                            .cloned(),
                        ..Default::default()
                    },
                )
                .await?;
        }

        for step_group in build.installation {
            type FutResult = Result<(), wings_api::client::ApiHttpError>;
            let mut tasks: Vec<Pin<Box<dyn Future<Output = FutResult> + Send>>> = Vec::new();
            tasks.reserve_exact(step_group.len());

            for step in step_group {
                match step {
                    crate::mcjars::MinecraftBuildInstallationStep::Download(download_step) => {
                        tasks.push(Box::pin(async {
                            api_client
                                .post_servers_server_files_pull(
                                    server.uuid,
                                    &wings_api::servers_server_files_pull::post::RequestBody {
                                        url: download_step.url,
                                        file_name: Some(download_step.file),
                                        foreground: true,
                                        use_header: false,
                                        root: "/".into(),
                                    },
                                )
                                .await?;

                            Ok(())
                        }));
                    }
                    crate::mcjars::MinecraftBuildInstallationStep::Unzip(unzip_step) => {
                        tasks.push(Box::pin(async {
                            api_client
                                .post_servers_server_files_decompress(server.uuid, &wings_api::servers_server_files_decompress::post::RequestBody {
                                    file: unzip_step.file,
                                    root: unzip_step.location,
                                    foreground: true
                                })
                                .await?;

                            Ok(())
                        }));
                    }
                    crate::mcjars::MinecraftBuildInstallationStep::Remove(remove_step) => {
                        tasks.push(Box::pin(async {
                            api_client
                                .post_servers_server_files_delete(
                                    server.uuid,
                                    &wings_api::servers_server_files_delete::post::RequestBody {
                                        files: vec![remove_step.location],
                                        root: "/".into(),
                                    },
                                )
                                .await?;

                            Ok(())
                        }));
                    }
                }
            }

            futures_util::future::try_join_all(tasks).await?;
        }

        let mut docker_image = None;
        for image in server.egg.docker_images.values() {
            let i = image
                .rfind(|c: char| !c.is_numeric())
                .map(|i| i + 1)
                .unwrap_or(0);
            let java_version: u8 = match image[i..].parse() {
                Ok(java_version) => java_version,
                Err(_) => continue,
            };

            if java_version == version.java {
                docker_image = Some(image);
                break;
            }
        }

        if let Some(docker_image) = docker_image.cloned() {
            server
                .update(
                    &state,
                    shared::models::server::UpdateServerOptions {
                        image: Some(docker_image),
                        ..Default::default()
                    },
                )
                .await?;
        }

        activity_logger
            .log(
                "server:version.install",
                serde_json::json!({
                    "type": build.r#type,
                    "version": build.version_id.or(build.project_version_id),
                    "build": build.name,
                    "truncate_directory": data.truncate_directory,
                    "accept_eula": data.accept_eula
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
