use super::State;
use utoipa_axum::{router::OpenApiRouter, routes};

mod get {
    use serde::Serialize;
    use shared::{
        GetState,
        models::user::GetPermissionManager,
        response::{ApiResponse, ApiResponseResult},
    };
    use utoipa::ToSchema;

    #[derive(ToSchema, Serialize)]
    struct Response<'a> {
        #[schema(inline)]
        settings: &'a crate::settings::ExtensionSettingsData,
    }

    #[utoipa::path(get, path = "/", responses(
        (status = OK, body = inline(Response)),
    ))]
    pub async fn route(state: GetState, permissions: GetPermissionManager) -> ApiResponseResult {
        permissions.has_admin_permission("settings.read")?;

        let settings = state.settings.get().await?;
        let extension_settings: &crate::settings::ExtensionSettingsData =
            settings.find_extension_settings()?;

        ApiResponse::new_serialized(Response {
            settings: extension_settings,
        })
        .ok()
    }
}

mod put {
    use axum::http::StatusCode;
    use garde::Validate;
    use indexmap::IndexMap;
    use serde::{Deserialize, Serialize};
    use shared::{
        ApiError, GetState,
        models::{admin_activity::GetAdminActivityLogger, user::GetPermissionManager},
        response::{ApiResponse, ApiResponseResult},
    };
    use utoipa::ToSchema;

    #[derive(ToSchema, Validate, Deserialize)]
    pub struct Payload {
        #[garde(url, length(chars, min = 1, max = 255))]
        #[schema(format = "url", min_length = 1, max_length = 255)]
        mcjars_url: Option<compact_str::CompactString>,
        #[garde(length(chars, min = 64, max = 64))]
        #[schema(min_length = 64, max_length = 64)]
        mcjars_api_key: Option<Option<compact_str::CompactString>>,
        #[garde(url, length(chars, min = 1, max = 255))]
        #[schema(format = "url", min_length = 1, max_length = 255)]
        mcjars_icon_base_url: Option<compact_str::CompactString>,
        #[garde(length(chars, min = 1))]
        #[schema(min_length = 1)]
        mcjars_icon_file_extension: Option<compact_str::CompactString>,

        #[garde(skip)]
        #[serde(with = "::serde_with::rust::double_option")]
        mcjars_type_order:
            Option<Option<IndexMap<compact_str::CompactString, Vec<compact_str::CompactString>>>>,

        #[garde(length(max = 100))]
        #[schema(max_items = 100)]
        type_egg_assignments: Option<Vec<crate::settings::EggAssignment>>,
        #[garde(skip)]
        collect_installation_statistics: Option<bool>,
    }

    #[derive(ToSchema, Serialize)]
    struct Response {}

    #[utoipa::path(put, path = "/", responses(
        (status = OK, body = inline(Response)),
    ), request_body = inline(Payload))]
    pub async fn route(
        state: GetState,
        permissions: GetPermissionManager,
        activity_logger: GetAdminActivityLogger,
        shared::Payload(data): shared::Payload<Payload>,
    ) -> ApiResponseResult {
        if let Err(errors) = shared::utils::validate_data(&data) {
            return ApiResponse::new_serialized(ApiError::new_strings_value(errors))
                .with_status(StatusCode::BAD_REQUEST)
                .ok();
        }

        permissions.has_admin_permission("extensions.configure")?;

        let mut settings = state.settings.get_mut().await?;
        let extension_settings =
            settings.find_mut_extension_settings::<crate::settings::ExtensionSettingsData>()?;

        if let Some(mcjars_url) = data.mcjars_url {
            extension_settings.mcjars_url = mcjars_url;
        }
        if let Some(mcjars_api_key) = data.mcjars_api_key {
            extension_settings.mcjars_api_key = mcjars_api_key;
        }
        if let Some(mcjars_icon_base_url) = data.mcjars_icon_base_url {
            extension_settings.mcjars_icon_base_url = mcjars_icon_base_url;
        }
        if let Some(mcjars_icon_file_extension) = data.mcjars_icon_file_extension {
            extension_settings.mcjars_icon_file_extension = mcjars_icon_file_extension;
        }
        if let Some(mcjars_type_order) = data.mcjars_type_order {
            extension_settings.mcjars_type_order = mcjars_type_order;
        }
        if let Some(type_egg_assignments) = data.type_egg_assignments {
            extension_settings.type_egg_assignments = type_egg_assignments;
        }
        if let Some(collect_installation_statistics) = data.collect_installation_statistics {
            extension_settings.collect_installation_statistics = collect_installation_statistics;
        }

        let mut extension_settings_json = serde_json::to_value(&extension_settings)?;
        settings.save().await?;

        if let Some(obj) = extension_settings_json.as_object_mut()
            && let Some(mcjars_api_key) = obj.get_mut("mcjars_api_key")
            && !mcjars_api_key.is_null()
        {
            *mcjars_api_key = serde_json::Value::String("***".to_string());
        }

        activity_logger
            .log(
                "settings:extensions:update",
                serde_json::json!({
                    "extension": "dev.0x7d8.minecraftversionchanger",
                    "settings": extension_settings_json,
                }),
            )
            .await;

        ApiResponse::new_serialized(Response {}).ok()
    }
}

pub fn router(state: &State) -> OpenApiRouter<State> {
    OpenApiRouter::new()
        .routes(routes!(get::route))
        .routes(routes!(put::route))
        .with_state(state.clone())
}
