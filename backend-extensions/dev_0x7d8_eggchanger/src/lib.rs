use shared::{
    State,
    extensions::{
        Extension, ExtensionPermissionsBuilder, ExtensionRouteBuilder, ExtensionUpdateInfo,
        settings::ExtensionSettingsDeserializer,
    },
};
use std::sync::Arc;

mod _2038;
mod routes;
mod settings;

#[derive(Default)]
pub struct ExtensionStruct;

#[async_trait::async_trait]
impl Extension for ExtensionStruct {
    async fn initialize_router(
        &mut self,
        state: State,
        builder: ExtensionRouteBuilder,
    ) -> ExtensionRouteBuilder {
        builder
            .add_admin_api_router(|routes| {
                routes.nest(
                    "/extensions/dev.0x7d8.eggchanger",
                    routes::admin::router(&state),
                )
            })
            .add_client_server_api_router(|routes| {
                routes.nest("/settings/egg-changer", routes::server::router(&state))
            })
            .add_global_router(|routes| {
                routes.route(
                    "/2038/dev.0x7d8.eggchanger",
                    axum::routing::get(_2038::route),
                )
            })
    }

    async fn initialize_permissions(
        &mut self,
        _state: State,
        builder: ExtensionPermissionsBuilder,
    ) -> ExtensionPermissionsBuilder {
        builder.mutate_server_permission_group("settings", |group| {
            group.add_permission("egg-changer", "Allows updating the egg of a server.")
        })
    }

    async fn settings_deserializer(&self, _state: State) -> ExtensionSettingsDeserializer {
        Arc::new(settings::ExtensionSettingsDataDeserializer)
    }

    async fn check_for_updates(
        &self,
        state: State,
        current_version: &semver::Version,
    ) -> Result<Option<ExtensionUpdateInfo>, anyhow::Error> {
        _2038::check_for_updates(&state, current_version).await
    }
}
