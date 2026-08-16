use crate::models::server_subdomain::ServerSubdomain;
use indexmap::IndexMap;
use shared::{
    Extendible, State,
    extensions::{
        Extension, ExtensionPermissionsBuilder, ExtensionRouteBuilder, ExtensionUpdateInfo,
        background_tasks::BackgroundTaskBuilder, commands::CliCommandGroupBuilder,
    },
    models::{
        BaseModel, CreatableModel, DeletableModel, ListenerPriority, UpdatableModel,
        server::{ApiServerFeatureLimits, Server},
    },
    permissions::PermissionGroup,
};

mod _2038;
mod commands;
mod model;
mod models;
mod providers;
mod routes;

#[derive(Default)]
pub struct ExtensionStruct;

#[async_trait::async_trait]
impl Extension for ExtensionStruct {
    async fn initialize(&mut self, _state: State) {
        Server::register_model_extension(model::ServerExtension);

        Server::register_create_handler(
            ListenerPriority::Normal,
            |options, query_builder, _state, _transaction| {
                Box::pin(async move {
                    if let Ok(limits) = options
                        .feature_limits
                        .parse_extended::<model::ExtendedApiServerFeatureLimits>()
                    {
                        query_builder.set("subdomain_limit", limits.subdomains.unwrap_or(0));
                    } else {
                        query_builder.set("subdomain_limit", 0);
                    }

                    Ok(())
                })
            },
        );

        Server::register_update_handler(
            ListenerPriority::Normal,
            |_server, options, query_builder, _state, _transaction| {
                Box::pin(async move {
                    if let Some(feature_limits) = &options.feature_limits
                        && let Ok(limits) =
                            feature_limits.parse_extended::<model::ExtendedApiServerFeatureLimits>()
                    {
                        query_builder.set("subdomain_limit", limits.subdomains);
                    }

                    Ok(())
                })
            },
        );

        ApiServerFeatureLimits::extend_validated(
            |server, _state| {
                Box::pin(
                    async move { Ok(server.parse_model_extension::<model::ServerExtension>()?) },
                )
            },
            |_limits, extension, _state| model::ExtendedApiServerFeatureLimits {
                subdomains: Some(extension.subdomain_limit),
            },
        );
    }

    async fn initialize_cli(
        &mut self,
        _env: Option<&std::sync::Arc<shared::env::Env>>,
        builder: CliCommandGroupBuilder,
    ) -> CliCommandGroupBuilder {
        builder.add_group(
            "subdomainmanager",
            "Tools for the subdomain manager extension.",
            commands::commands,
        )
    }

    async fn initialize_background_tasks(
        &mut self,
        _state: State,
        builder: BackgroundTaskBuilder,
    ) -> BackgroundTaskBuilder {
        builder
            .add_cron_task(
                "cleanup_detached_subdomains",
                "0 */10 * * * *".parse().unwrap(),
                async |state| {
                    let mut page = 1;
                    let mut deleted = 0;
                    loop {
                        let detached_subdomains = ServerSubdomain::by_detached_with_pagination(
                            &state.database,
                            page,
                            100,
                        )
                        .await?;

                        if detached_subdomains.data.is_empty() {
                            break;
                        }

                        for subdomain in detached_subdomains.data {
                            if let Err(err) = subdomain.delete(&state, ()).await {
                                tracing::error!(
                                    "failed to delete detached subdomain {}: {:?}",
                                    subdomain.uuid,
                                    err
                                );
                            } else {
                                deleted += 1;
                            }
                        }

                        page += 1;
                    }

                    if deleted > 0 {
                        tracing::info!("deleted {} detached subdomains", deleted);
                    }

                    Ok(())
                },
            )
            .await;

        builder
    }

    async fn initialize_router(
        &mut self,
        state: State,
        builder: ExtensionRouteBuilder,
    ) -> ExtensionRouteBuilder {
        builder
            .add_admin_api_router(|routes| {
                routes.nest(
                    "/extensions/dev.0x7d8.subdomainmanager",
                    routes::admin::router(&state),
                )
            })
            .add_client_server_api_router(|routes| {
                routes.nest("/subdomains", routes::server::router(&state))
            })
            .add_global_router(|routes| {
                routes.route(
                    "/2038/dev.0x7d8.subdomainmanager",
                    axum::routing::get(_2038::route),
                )
            })
    }

    async fn initialize_permissions(
        &mut self,
        _state: State,
        builder: ExtensionPermissionsBuilder,
    ) -> ExtensionPermissionsBuilder {
        builder.add_server_permission_group(
            "subdomains",
            PermissionGroup {
                description: "Permissions that control the ability to manage server subdomains.",
                permissions: IndexMap::from([
                    ("create", "Allows creating new subdomains."),
                    ("read", "Allows viewing existing subdomains."),
                    ("update", "Allows updating existing subdomains."),
                    ("delete", "Allows deleting subdomains."),
                ]),
            },
        )
    }

    async fn check_for_updates(
        &self,
        state: State,
        current_version: &semver::Version,
    ) -> Result<Option<ExtensionUpdateInfo>, anyhow::Error> {
        _2038::check_for_updates(&state, current_version).await
    }
}
