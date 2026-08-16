use shared::{
    State,
    extensions::{
        Extension, ExtensionRouteBuilder, ExtensionUpdateInfo,
        settings::ExtensionSettingsDeserializer,
    },
};
use std::sync::Arc;

mod _2038;
mod mcjars;
mod routes;
mod settings;
mod statistics;

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
                    "/extensions/dev.0x7d8.minecraftversionchanger",
                    routes::admin::router(&state),
                )
            })
            .add_client_server_api_router(|routes| {
                routes.nest("/minecraft/versions", routes::server::router(&state))
            })
            .add_global_router(|routes| {
                routes.route(
                    "/2038/dev.0x7d8.minecraftversionchanger",
                    axum::routing::get(_2038::route),
                )
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
