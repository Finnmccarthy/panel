use super::State;
use utoipa_axum::{router::OpenApiRouter, routes};

mod _type_;

mod get {
    use indexmap::IndexMap;
    use serde::Serialize;
    use shared::{
        GetState,
        models::user::GetPermissionManager,
        response::{ApiResponse, ApiResponseResult},
    };
    use utoipa::ToSchema;

    #[derive(ToSchema, Serialize)]
    struct Response {
        types: IndexMap<
            compact_str::CompactString,
            IndexMap<compact_str::CompactString, crate::mcjars::MinecraftType>,
        >,
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
    pub async fn route(state: GetState, permissions: GetPermissionManager) -> ApiResponseResult {
        permissions.has_server_permission("files.read")?;

        let types = state
            .cache
            .cached("minecraft::mcjars::types", 60 * 30, || async {
                crate::mcjars::types(
                    crate::mcjars::ApiContext::from_settings(&state.settings).await?,
                    &state,
                )
                .await
            })
            .await?;

        let settings = state.settings.get().await?;
        let extension_settings: &crate::settings::ExtensionSettingsData =
            settings.find_extension_settings()?;

        let mut ordered_types = IndexMap::new();
        let flat_types = types
            .iter()
            .flat_map(|(_, types)| {
                types
                    .iter()
                    .map(move |(type_id, type_data)| (type_id.clone(), type_data))
            })
            .collect::<Vec<_>>();

        if let Some(mcjars_type_order) = &extension_settings.mcjars_type_order {
            for (category, type_list) in mcjars_type_order.iter() {
                let mut ordered_type_map = IndexMap::new();
                for type_id in type_list {
                    if let Some((type_identifier, type_data)) =
                        flat_types.iter().find(|(id, _)| id == type_id)
                    {
                        let mut type_data = (*type_data).clone();
                        type_data.icon = compact_str::format_compact!(
                            "{}/{}.{}",
                            extension_settings
                                .mcjars_icon_base_url
                                .trim_end_matches('/'),
                            type_identifier.to_lowercase(),
                            extension_settings
                                .mcjars_icon_file_extension
                                .trim_start_matches('.')
                        );

                        ordered_type_map.insert(type_id.clone(), type_data);
                    }
                }
                ordered_types.insert(category.clone(), ordered_type_map);
            }
        } else {
            for (mut category, types) in types {
                let mut ordered_type_map = IndexMap::new();
                for (type_id, mut type_data) in types {
                    type_data.icon = compact_str::format_compact!(
                        "{}/{}.{}",
                        extension_settings
                            .mcjars_icon_base_url
                            .trim_end_matches('/'),
                        type_id.to_lowercase(),
                        extension_settings
                            .mcjars_icon_file_extension
                            .trim_start_matches('.')
                    );

                    ordered_type_map.insert(type_id, type_data);
                }

                category[0..1].make_ascii_uppercase();
                ordered_types.insert(category, ordered_type_map);
            }
        }

        ApiResponse::new_serialized(Response {
            types: ordered_types,
        })
        .ok()
    }
}

pub fn router(state: &State) -> OpenApiRouter<State> {
    OpenApiRouter::new()
        .routes(routes!(get::route))
        .nest("/{type}", _type_::router(state))
        .with_state(state.clone())
}
