use super::State;
use utoipa_axum::{router::OpenApiRouter, routes};

mod get {
    use std::collections::BTreeMap;

    use futures_util::StreamExt;
    use serde::Serialize;
    use shared::{
        GetState,
        models::{
            ByUuid, IntoApiObject, nest_egg::NestEgg, server::GetServer, user::GetPermissionManager,
        },
        response::{ApiResponse, ApiResponseResult},
    };
    use utoipa::ToSchema;

    #[derive(ToSchema, Serialize)]
    struct ResponseEggGroup {
        name: compact_str::CompactString,
        name_translations: BTreeMap<compact_str::CompactString, compact_str::CompactString>,
        eggs: Vec<shared::models::nest_egg::ApiNestEgg>,

        force_update_startup: bool,
        force_reinstall: bool,
        force_reinstall_truncate_files: bool,
    }

    #[derive(ToSchema, Serialize)]
    struct Response {
        #[schema(inline)]
        egg_groups: Vec<ResponseEggGroup>,
    }

    #[utoipa::path(get, path = "/", responses(
        (status = OK, body = inline(Response)),
    ), params(
        (
            "server" = uuid::Uuid,
            description = "The server ID",
            example = "123e4567-e89b-12d3-a456-426614174000",
        ),
    ))]
    pub async fn route(
        state: GetState,
        permissions: GetPermissionManager,
        server: GetServer,
    ) -> ApiResponseResult {
        permissions.has_server_permission("settings.egg-changer")?;

        let settings = state.settings.get().await?;
        let extension_settings: &crate::settings::ExtensionSettingsData =
            settings.find_extension_settings()?;

        struct PendingEggGroup<F: Future<Output = Result<Option<NestEgg>, anyhow::Error>>> {
            name: compact_str::CompactString,
            name_translations: BTreeMap<compact_str::CompactString, compact_str::CompactString>,
            eggs: Vec<F>,

            force_update_startup: bool,
            force_reinstall: bool,
            force_reinstall_truncate_files: bool,
        }

        let mut egg_groups = Vec::new();
        for group in &extension_settings.egg_groups {
            if !group.affected_eggs.contains(&server.egg.uuid) {
                continue;
            }

            let mut eggs = Vec::new();

            for egg_uuid in &group.eggs {
                let state = state.clone();
                let egg_uuid = *egg_uuid;

                eggs.push(async move {
                    NestEgg::by_uuid_optional_cached(&state.database, egg_uuid).await
                });
            }

            egg_groups.push(PendingEggGroup {
                name: group.name.clone(),
                name_translations: group.name_translations.clone(),
                eggs,

                force_update_startup: group.force_update_startup,
                force_reinstall: group.force_reinstall,
                force_reinstall_truncate_files: group.force_reinstall_truncate_files,
            });
        }
        drop(settings);

        let mut response_egg_groups = Vec::new();
        for group in egg_groups {
            let mut response_eggs = Vec::new();

            let mut eggs = futures_util::stream::iter(group.eggs).buffered(10);
            while let Some(egg) = eggs.next().await {
                if let Some(egg) = egg? {
                    response_eggs.push(egg.into_api_object(&state, ()).await?);
                }
            }

            response_egg_groups.push(ResponseEggGroup {
                name: group.name,
                name_translations: group.name_translations,
                eggs: response_eggs,

                force_update_startup: group.force_update_startup,
                force_reinstall: group.force_reinstall,
                force_reinstall_truncate_files: group.force_reinstall_truncate_files,
            });
        }

        ApiResponse::new_serialized(Response {
            egg_groups: response_egg_groups,
        })
        .ok()
    }
}

pub fn router(state: &State) -> OpenApiRouter<State> {
    OpenApiRouter::new()
        .routes(routes!(get::route))
        .with_state(state.clone())
}
