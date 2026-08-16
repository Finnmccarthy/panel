use axum::http::HeaderMap;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use shared::State;
use utoipa::ToSchema;

pub struct ApiContext {
    pub api_url: compact_str::CompactString,
    pub api_key: Option<compact_str::CompactString>,
}

impl ApiContext {
    pub async fn from_settings(
        settings: &shared::settings::Settings,
    ) -> Result<Self, anyhow::Error> {
        let settings = settings.get().await?;
        let ext_settings: &super::settings::ExtensionSettingsData =
            settings.find_extension_settings()?;

        Ok(ApiContext {
            api_url: ext_settings.mcjars_url.clone(),
            api_key: ext_settings.mcjars_api_key.clone(),
        })
    }

    pub fn into_headers(self) -> HeaderMap {
        let mut headers = HeaderMap::new();

        if let Some(api_key) = &self.api_key
            && let Ok(api_key) = api_key.parse()
        {
            headers.insert("Authorization", api_key);
        }

        headers
    }
}

#[derive(ToSchema, Deserialize, Serialize, Clone, Copy)]
pub struct MinecraftTypeVersions {
    pub minecraft: u32,
    pub project: u32,
}

#[derive(ToSchema, Deserialize, Serialize, Clone)]
pub struct MinecraftType {
    pub name: compact_str::CompactString,
    pub icon: compact_str::CompactString,
    pub color: compact_str::CompactString,
    pub homepage: compact_str::CompactString,
    pub deprecated: bool,
    pub experimental: bool,
    pub description: compact_str::CompactString,
    pub builds: u32,
    #[schema(inline)]
    pub versions: MinecraftTypeVersions,
}

#[derive(ToSchema, Deserialize, Serialize)]
pub struct MinecraftVersion {
    pub id: compact_str::CompactString,
    pub r#type: compact_str::CompactString,
    pub supported: bool,
    pub java: u8,
    pub builds: u32,
    pub created: chrono::DateTime<chrono::Utc>,

    pub latest: MinecraftBuild,
}

#[derive(ToSchema, Deserialize, Serialize)]
pub struct MinimalMinecraftVersion {
    pub r#type: compact_str::CompactString,
    pub supported: bool,
    pub java: u8,
    pub builds: u32,
    pub created: chrono::DateTime<chrono::Utc>,
}

#[derive(ToSchema, Deserialize, Serialize)]
pub struct MinecraftBuild {
    pub uuid: uuid::Uuid,
    pub r#type: compact_str::CompactString,
    pub experimental: bool,
    pub name: compact_str::CompactString,

    pub version_id: Option<compact_str::CompactString>,
    pub project_version_id: Option<compact_str::CompactString>,

    pub installation: Vec<Vec<MinecraftBuildInstallationStep>>,

    pub created: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(ToSchema, Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum MinecraftBuildInstallationStep {
    #[serde(rename = "download")]
    Download(#[schema(inline)] InstallationStepDownload),
    #[serde(rename = "unzip")]
    Unzip(#[schema(inline)] InstallationStepUnzip),
    #[serde(rename = "remove")]
    Remove(#[schema(inline)] InstallationStepRemove),
}

#[derive(ToSchema, Deserialize, Serialize)]
pub struct InstallationStepDownload {
    pub url: compact_str::CompactString,
    pub file: compact_str::CompactString,
    pub size: u64,
}
#[derive(ToSchema, Deserialize, Serialize)]
pub struct InstallationStepUnzip {
    pub file: compact_str::CompactString,
    pub location: compact_str::CompactString,
}
#[derive(ToSchema, Deserialize, Serialize)]
pub struct InstallationStepRemove {
    pub location: compact_str::CompactString,
}

pub async fn lookup_sha256(
    ctx: ApiContext,
    state: &State,
    hash: &str,
) -> Result<[MinecraftBuild; 2], anyhow::Error> {
    let res = state
        .client
        .get(format!("{}/api/v3/builds/{hash}", ctx.api_url))
        .headers(ctx.into_headers())
        .send()
        .await?;
    let data: ApiResponse = res.json().await?;

    #[derive(Deserialize)]
    struct ApiResponse {
        build: MinecraftBuild,
        latest: MinecraftBuild,
    }

    Ok([data.build, data.latest])
}

pub async fn lookup_uuid(
    ctx: ApiContext,
    state: &State,
    uuid: uuid::Uuid,
) -> Result<(MinecraftBuild, MinimalMinecraftVersion), anyhow::Error> {
    let res = state
        .client
        .get(format!("{}/api/v3/builds/{uuid}", ctx.api_url))
        .headers(ctx.into_headers())
        .send()
        .await?;
    let data: ApiResponse = res.json().await?;

    #[derive(Deserialize)]
    struct ApiResponse {
        build: MinecraftBuild,
        version: MinimalMinecraftVersion,
    }

    Ok((data.build, data.version))
}

pub async fn types(
    ctx: ApiContext,
    state: &State,
) -> Result<
    IndexMap<compact_str::CompactString, IndexMap<compact_str::CompactString, MinecraftType>>,
    anyhow::Error,
> {
    let res = state
        .client
        .get(format!("{}/api/v2/types", ctx.api_url))
        .headers(ctx.into_headers())
        .send()
        .await?;
    let data: ApiResponse = res.json().await?;

    #[derive(Deserialize)]
    struct ApiResponse {
        types: IndexMap<
            compact_str::CompactString,
            IndexMap<compact_str::CompactString, MinecraftType>,
        >,
    }

    Ok(data.types)
}

pub async fn versions(
    ctx: ApiContext,
    state: &State,
    type_identifier: &str,
    page: i64,
    per_page: i64,
    search: Option<&str>,
) -> Result<shared::models::Pagination<MinecraftVersion>, anyhow::Error> {
    let res = state
        .client
        .get(format!(
            "{}/api/v3/builds/types/{}/versions",
            ctx.api_url, type_identifier,
        ))
        .query(&[
            ("page", page.to_string().as_str()),
            ("per_page", per_page.to_string().as_str()),
            ("search", search.unwrap_or("")),
        ])
        .headers(ctx.into_headers())
        .send()
        .await?;
    let data = res.text().await?;

    println!("MCJARS API response: {}", data);

    let data = serde_json::from_str::<ApiResponse>(&data)?;

    #[derive(Deserialize)]
    struct ApiResponsePagination {
        total: i64,
        per_page: i64,
        page: i64,
        data: Vec<MinecraftVersion>,
    }

    #[derive(Deserialize)]
    struct ApiResponse {
        versions: ApiResponsePagination,
    }

    Ok(shared::models::Pagination {
        total: data.versions.total,
        per_page: data.versions.per_page,
        page: data.versions.page,
        data: data.versions.data,
    })
}

pub async fn builds(
    ctx: ApiContext,
    state: &State,
    type_identifier: &str,
    version_identifier: &str,
    page: i64,
    per_page: i64,
    search: Option<&str>,
) -> Result<shared::models::Pagination<MinecraftBuild>, anyhow::Error> {
    let res = state
        .client
        .get(format!(
            "{}/api/v3/builds/types/{}/versions/{}",
            ctx.api_url, type_identifier, version_identifier
        ))
        .query(&[
            ("page", page.to_string().as_str()),
            ("per_page", per_page.to_string().as_str()),
            ("search", search.unwrap_or("")),
        ])
        .headers(ctx.into_headers())
        .send()
        .await?;
    let data: ApiResponse = res.json().await?;

    #[derive(Deserialize)]
    struct ApiResponsePagination {
        total: i64,
        per_page: i64,
        page: i64,
        data: Vec<MinecraftBuild>,
    }

    #[derive(Deserialize)]
    struct ApiResponse {
        builds: ApiResponsePagination,
    }

    Ok(shared::models::Pagination {
        total: data.builds.total,
        per_page: data.builds.per_page,
        page: data.builds.page,
        data: data.builds.data,
    })
}
