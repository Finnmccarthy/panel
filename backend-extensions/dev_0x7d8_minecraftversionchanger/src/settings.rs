use compact_str::ToCompactString;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use shared::{
    extensions::settings::{
        ExtensionSettings, SettingsDeserializeExt, SettingsDeserializer, SettingsSerializeExt,
        SettingsSerializer,
    },
    prelude::StringExt,
};
use std::collections::HashSet;
use utoipa::ToSchema;

#[derive(ToSchema, Serialize, Deserialize)]
pub struct EggAssignment {
    pub affected_types: HashSet<compact_str::CompactString>,
    pub egg: uuid::Uuid,
}

#[derive(ToSchema, Serialize, Deserialize)]
pub struct ExtensionSettingsData {
    pub mcjars_url: compact_str::CompactString,
    pub mcjars_api_key: Option<compact_str::CompactString>,
    pub mcjars_icon_base_url: compact_str::CompactString,
    pub mcjars_icon_file_extension: compact_str::CompactString,

    pub mcjars_type_order:
        Option<IndexMap<compact_str::CompactString, Vec<compact_str::CompactString>>>,

    pub type_egg_assignments: Vec<EggAssignment>,
    pub collect_installation_statistics: bool,
}

#[async_trait::async_trait]
impl SettingsSerializeExt for ExtensionSettingsData {
    async fn serialize(
        &self,
        serializer: SettingsSerializer,
    ) -> Result<SettingsSerializer, anyhow::Error> {
        Ok(serializer
            .write_raw_setting("mcjars_url", &*self.mcjars_url)
            .write_raw_setting(
                "mcjars_api_key",
                self.mcjars_api_key.as_deref().unwrap_or(""),
            )
            .write_raw_setting("mcjars_icon_base_url", &*self.mcjars_icon_base_url)
            .write_raw_setting(
                "mcjars_icon_file_extension",
                &*self.mcjars_icon_file_extension,
            )
            .write_serde_setting("mcjars_type_order", &self.mcjars_type_order)?
            .write_serde_setting("type_egg_assignments", &self.type_egg_assignments)?
            .write_raw_setting(
                "collect_installation_statistics",
                self.collect_installation_statistics.to_compact_string(),
            ))
    }
}

pub struct ExtensionSettingsDataDeserializer;

#[async_trait::async_trait]
impl SettingsDeserializeExt for ExtensionSettingsDataDeserializer {
    async fn deserialize_boxed(
        &self,
        mut deserializer: SettingsDeserializer<'_>,
    ) -> Result<ExtensionSettings, anyhow::Error> {
        Ok(Box::new(ExtensionSettingsData {
            mcjars_url: deserializer
                .take_raw_setting("mcjars_url")
                .unwrap_or_else(|| "https://mcjars.app".into()),
            mcjars_api_key: deserializer
                .take_raw_setting("mcjars_api_key")
                .and_then(|s| s.into_optional()),
            mcjars_icon_base_url: deserializer
                .take_raw_setting("mcjars_icon_base_url")
                .unwrap_or_else(|| "https://s3.mcjars.app/icons/".into()),
            mcjars_icon_file_extension: deserializer
                .take_raw_setting("mcjars_icon_file_extension")
                .unwrap_or_else(|| "png".into()),
            mcjars_type_order: deserializer
                .read_serde_setting("mcjars_type_order")
                .unwrap_or(None),
            type_egg_assignments: deserializer
                .read_serde_setting("type_egg_assignments")
                .unwrap_or_default(),
            collect_installation_statistics: deserializer
                .take_raw_setting("collect_installation_statistics")
                .and_then(|s| s.parse().ok())
                .unwrap_or(true),
        }))
    }
}
