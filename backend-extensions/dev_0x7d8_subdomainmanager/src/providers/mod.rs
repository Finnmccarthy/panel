use compact_str::ToCompactString;
use serde::{Deserialize, Serialize};
use sqlx::prelude::Type;
use utoipa::ToSchema;

pub mod cloudflare;

#[async_trait::async_trait]
pub trait DnsProvider: Send + Sync {
    fn validate_config(&self, provider_config: &serde_json::Value) -> Result<(), garde::Report>;
    fn censor_config(&self, provider_config: &mut serde_json::Value);

    async fn create_records(
        &self,
        provider_config: &serde_json::Value,
        params: PlaceholderParams<'_>,
    ) -> Result<(), anyhow::Error>;

    async fn update_records(
        &self,
        provider_config: &serde_json::Value,
        params: PlaceholderParams<'_>,
    ) -> Result<(), anyhow::Error>;

    async fn delete_records(
        &self,
        provider_config: &serde_json::Value,
        record_uuid: uuid::Uuid,
    ) -> Result<(), anyhow::Error>;

    async fn test(&self, provider_config: &serde_json::Value) -> Result<bool, anyhow::Error>;
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, ToSchema, Type)]
#[sqlx(
    type_name = "dev_0x7d8_subdomainmanager_provider",
    rename_all = "SCREAMING_SNAKE_CASE"
)]
#[serde(rename_all = "lowercase")]
pub enum Provider {
    Cloudflare,
}

impl Provider {
    pub fn build(&self, client: reqwest::Client) -> Box<dyn DnsProvider> {
        match self {
            Self::Cloudflare => Box::new(cloudflare::CloudflareProvider::new(client)),
        }
    }
}

pub struct PlaceholderParams<'a> {
    pub subdomain: &'a crate::models::server_subdomain::ServerSubdomain,
    pub allocation: &'a shared::models::server_allocation::ServerAllocation,
}

impl PlaceholderParams<'_> {
    pub fn comment(&self) -> String {
        format!(
            "dev.0x7d8.subdomainmanager.record_uuid={}",
            self.subdomain.uuid
        )
    }
}

pub async fn apply_placeholders(
    text: &str,
    p: PlaceholderParams<'_>,
) -> Result<String, anyhow::Error> {
    let ip_alias = p
        .allocation
        .allocation
        .ip_alias
        .clone()
        .unwrap_or_else(|| p.allocation.allocation.ip.ip().to_compact_string());

    let resolved_ip_alias = if text.contains("{ip_alias_forceip}") {
        let mut resolved: Vec<_> = tokio::net::lookup_host((ip_alias.as_str(), 0))
            .await?
            .collect();

        resolved.sort_by_key(|addr| match addr {
            std::net::SocketAddr::V4(_) => 0,
            std::net::SocketAddr::V6(_) => 1,
        });

        resolved
            .into_iter()
            .next()
            .ok_or_else(|| anyhow::anyhow!("failed to resolve IP alias"))?
            .ip()
            .to_compact_string()
    } else {
        "".into()
    };

    Ok(text
        .replace("{subdomain}", &p.subdomain.subdomain)
        .replace("{domain}", &p.subdomain.domain.domain)
        .replace("{ip_alias_forceip}", &resolved_ip_alias)
        .replace("{ip_alias}", &ip_alias)
        .replace("{ip}", &p.allocation.allocation.ip.ip().to_compact_string())
        .replace("{port}", &p.allocation.allocation.port.to_compact_string())
        .replace("{comment}", &p.comment()))
}
