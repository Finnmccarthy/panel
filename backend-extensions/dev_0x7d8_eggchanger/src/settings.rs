use garde::Validate;
use indexmap::IndexSet;
use serde::{Deserialize, Serialize};
use shared::extensions::settings::{
    ExtensionSettings, SettingsDeserializeExt, SettingsDeserializer, SettingsSerializeExt,
    SettingsSerializer,
};
use std::collections::{BTreeMap, HashSet};
use utoipa::ToSchema;

#[derive(ToSchema, Validate, Serialize, Deserialize)]
pub struct EggChangerEggGroup {
    #[garde(length(chars, min = 1, max = 255))]
    #[schema(min_length = 1, max_length = 255)]
    pub name: compact_str::CompactString,
    #[garde(skip)]
    pub name_translations: BTreeMap<compact_str::CompactString, compact_str::CompactString>,
    #[garde(skip)]
    pub eggs: IndexSet<uuid::Uuid>,

    #[garde(skip)]
    pub force_update_startup: bool,
    #[garde(skip)]
    pub force_reinstall: bool,
    #[garde(skip)]
    pub force_reinstall_truncate_files: bool,

    #[garde(skip)]
    #[serde(default)]
    pub reassign_allocations: bool,

    #[garde(skip)]
    pub affected_eggs: HashSet<uuid::Uuid>,
}

#[derive(ToSchema, Serialize, Deserialize)]
pub struct ExtensionSettingsData {
    pub egg_groups: Vec<EggChangerEggGroup>,
}

#[async_trait::async_trait]
impl SettingsSerializeExt for ExtensionSettingsData {
    async fn serialize(
        &self,
        serializer: SettingsSerializer,
    ) -> Result<SettingsSerializer, anyhow::Error> {
        Ok(serializer.write_serde_setting("egg_groups", &self.egg_groups)?)
    }
}

pub struct ExtensionSettingsDataDeserializer;

#[async_trait::async_trait]
impl SettingsDeserializeExt for ExtensionSettingsDataDeserializer {
    async fn deserialize_boxed(
        &self,
        deserializer: SettingsDeserializer<'_>,
    ) -> Result<ExtensionSettings, anyhow::Error> {
        Ok(Box::new(ExtensionSettingsData {
            egg_groups: deserializer
                .read_serde_setting("egg_groups")
                .unwrap_or_else(|_| Vec::new()),
        }))
    }
}
