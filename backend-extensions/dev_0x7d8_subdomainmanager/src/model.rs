use garde::Validate;
use serde::{Deserialize, Serialize};
use shared::models::{ModelExtension, SafeModelExtension};
use sqlx::{Row, postgres::PgRow};
use std::collections::BTreeMap;
use utoipa::ToSchema;

#[derive(ToSchema, Validate, Serialize, Deserialize)]
pub struct ExtendedApiServerFeatureLimits {
    #[garde(range(min = 0))]
    #[schema(minimum = 0)]
    pub subdomains: Option<i32>,
}

#[derive(Serialize, Deserialize)]
pub struct ServerExtensionData {
    pub subdomain_limit: i32,
}

pub struct ServerExtension;

impl SafeModelExtension for ServerExtension {
    type Value = ServerExtensionData;

    fn name() -> &'static str {
        ServerExtension.extension_name()
    }
}

impl ModelExtension for ServerExtension {
    fn extension_name(&self) -> &'static str {
        "dev.0x7d8.subdomainmanager"
    }

    fn extended_columns(&self, prefix: &str) -> BTreeMap<&'static str, compact_str::CompactString> {
        BTreeMap::from([(
            "servers.subdomain_limit",
            compact_str::format_compact!("{prefix}subdomain_limit"),
        )])
    }

    fn map_extended(
        &self,
        prefix: &str,
        row: &PgRow,
    ) -> Result<shared::models::ModelExtensionMapType, shared::database::DatabaseError> {
        Ok(Box::new(ServerExtensionData {
            subdomain_limit: row
                .try_get(compact_str::format_compact!("{prefix}subdomain_limit").as_str())?,
        }))
    }
}
