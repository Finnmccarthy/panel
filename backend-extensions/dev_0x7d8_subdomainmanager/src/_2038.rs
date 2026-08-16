use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use shared::{
    extensions::ExtensionUpdateInfo,
    response::{ApiResponse, ApiResponseResult},
};
use utoipa::ToSchema;

pub const EXTENSION_ID: i32 = 20;

pub static USER: &str = "206595";
pub static TIMESTAMP: &str = "1786791666";
pub static NONCE: &str = "d02565a5e87b0f8d0a88e921dd5a324d";

#[derive(ToSchema, Serialize, Deserialize)]
pub struct ExtensionInfoProduct {
    pub version: compact_str::CompactString,
}

#[derive(ToSchema, Serialize, Deserialize)]
pub struct ExtensionInfoProvider {
    pub name: compact_str::CompactString,
    #[schema(format = "url")]
    pub link: compact_str::CompactString,
}

#[derive(ToSchema, Serialize, Deserialize)]
pub struct ExtensionInfoChangelog {
    pub content: compact_str::CompactString,
    pub created: chrono::DateTime<chrono::Utc>,
}

#[derive(ToSchema, Serialize, Deserialize)]
pub struct ExtensionInfo {
    #[schema(inline)]
    pub product: ExtensionInfoProduct,
    #[schema(inline)]
    pub providers: IndexMap<compact_str::CompactString, ExtensionInfoProvider>,
    #[schema(inline)]
    pub changelogs: IndexMap<compact_str::CompactString, ExtensionInfoChangelog>,
}

pub async fn get_extension_info(state: &shared::State) -> Result<ExtensionInfo, anyhow::Error> {
    state
        .cache
        .cached(
            &format!("2038::extension_info::{EXTENSION_ID}"),
            60 * 30,
            || async {
                let extension_info: ExtensionInfo = state
                    .client
                    .get(format!("https://api.2038.buzz/products/{EXTENSION_ID}"))
                    .send()
                    .await?
                    .json()
                    .await?;

                Ok::<_, anyhow::Error>(extension_info)
            },
        )
        .await
}

pub async fn check_for_updates(
    state: &shared::State,
    current_version: &semver::Version,
) -> Result<Option<ExtensionUpdateInfo>, anyhow::Error> {
    let extension_info = get_extension_info(state).await?;

    let latest_version = semver::Version::parse(&extension_info.product.version)?;

    if latest_version > *current_version {
        let mut changes = Vec::new();
        let mut versioned_changelogs = extension_info
            .changelogs
            .into_iter()
            .filter_map(|(version, changelog)| {
                let parsed_version = semver::Version::parse(&version).ok()?;

                if parsed_version > *current_version {
                    Some((parsed_version, changelog))
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        versioned_changelogs.sort_by(|a, b| a.0.cmp(&b.0));

        for (version, changelog) in versioned_changelogs {
            changes.push(compact_str::format_compact!(
                "{version}: {}",
                changelog.content
            ));
        }

        Ok(Some(ExtensionUpdateInfo {
            version: latest_version,
            changes,
        }))
    } else {
        Ok(None)
    }
}

pub async fn route() -> ApiResponseResult {
    #[derive(Serialize)]
    struct Response {
        nonce: &'static str,
        user: &'static str,
        timestamp: &'static str,
    }

    ApiResponse::new_serialized(Response {
        nonce: NONCE,
        user: USER,
        timestamp: TIMESTAMP,
    })
    .ok()
}
