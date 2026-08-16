use crate::providers::DnsProvider;
use crate::providers::{PlaceholderParams, apply_placeholders};
use compact_str::CompactString;
use garde::Validate;
use serde::Deserialize;

#[derive(Deserialize, Validate)]
#[garde(context(()))]
struct CloudflareConfig {
    #[garde(length(chars, min = 1))]
    #[serde(default)]
    token: CompactString,
    #[garde(length(chars, min = 1))]
    #[serde(default)]
    zone_id: CompactString,
    #[garde(skip)]
    api_data: serde_json::Value,
}

pub struct CloudflareProvider {
    client: reqwest::Client,
}

impl CloudflareProvider {
    pub fn new(client: reqwest::Client) -> Self {
        Self { client }
    }

    fn zone_url(zone_id: &str) -> String {
        format!("https://api.cloudflare.com/client/v4/zones/{zone_id}/dns_records")
    }

    fn comment_tag(record_uuid: uuid::Uuid) -> String {
        format!("dev.0x7d8.subdomainmanager.record_uuid={record_uuid}")
    }

    async fn list_record_ids(
        &self,
        token: &str,
        zone_id: &str,
        record_uuid: uuid::Uuid,
    ) -> Result<Vec<String>, anyhow::Error> {
        let res = self
            .client
            .get(Self::zone_url(zone_id))
            .bearer_auth(token)
            .query(&[("comment", &Self::comment_tag(record_uuid))])
            .send()
            .await?;

        if !res.status().is_success() {
            anyhow::bail!("failed to list DNS records: {}", res.text().await?);
        }

        #[derive(Deserialize)]
        struct ListResponse {
            result: Vec<serde_json::Value>,
        }

        let body: ListResponse = res.json().await?;
        Ok(body
            .result
            .into_iter()
            .filter_map(|r| r.get("id")?.as_str().map(|s| s.to_string()))
            .collect())
    }

    async fn batch(
        &self,
        token: &str,
        zone_id: &str,
        payload: serde_json::Value,
    ) -> Result<(), anyhow::Error> {
        let res = self
            .client
            .post(format!("{}/batch", Self::zone_url(zone_id)))
            .bearer_auth(token)
            .json(&payload)
            .send()
            .await?;

        if !res.status().is_success() {
            anyhow::bail!("failed to batch DNS records: {}", res.text().await?);
        }

        Ok(())
    }

    async fn substitute(
        api_data: &serde_json::Value,
        params: PlaceholderParams<'_>,
    ) -> Result<serde_json::Value, anyhow::Error> {
        let raw = serde_json::to_string(api_data)?;
        Ok(serde_json::from_str(
            &apply_placeholders(&raw, params).await?,
        )?)
    }
}

#[async_trait::async_trait]
impl DnsProvider for CloudflareProvider {
    fn validate_config(&self, provider_config: &serde_json::Value) -> Result<(), garde::Report> {
        let config: CloudflareConfig = serde_json::from_value(provider_config.clone())
            .unwrap_or_else(|_| CloudflareConfig {
                token: CompactString::default(),
                zone_id: CompactString::default(),
                api_data: serde_json::Value::Null,
            });
        config.validate()
    }

    fn censor_config(&self, provider_config: &mut serde_json::Value) {
        if let Some(obj) = provider_config.as_object_mut()
            && obj.contains_key("token")
        {
            obj.insert(
                "token".to_string(),
                serde_json::Value::String("***".to_string()),
            );
        }
    }

    async fn create_records(
        &self,
        provider_config: &serde_json::Value,
        params: PlaceholderParams<'_>,
    ) -> Result<(), anyhow::Error> {
        let config: CloudflareConfig = serde_json::from_value(provider_config.clone())?;
        let payload = Self::substitute(&config.api_data, params).await?;
        self.batch(&config.token, &config.zone_id, payload).await
    }

    async fn update_records(
        &self,
        provider_config: &serde_json::Value,
        params: PlaceholderParams<'_>,
    ) -> Result<(), anyhow::Error> {
        let config: CloudflareConfig = serde_json::from_value(provider_config.clone())?;

        let existing_ids = self
            .list_record_ids(&config.token, &config.zone_id, params.subdomain.uuid)
            .await?;

        let mut payload = Self::substitute(&config.api_data, params).await?;

        if !existing_ids.is_empty() {
            let deletes: Vec<serde_json::Value> = existing_ids
                .into_iter()
                .map(|id| serde_json::json!({ "id": id }))
                .collect();

            if let Some(obj) = payload.as_object_mut() {
                obj.insert("deletes".to_string(), serde_json::Value::Array(deletes));
            }
        }

        self.batch(&config.token, &config.zone_id, payload).await
    }

    async fn delete_records(
        &self,
        provider_config: &serde_json::Value,
        record_uuid: uuid::Uuid,
    ) -> Result<(), anyhow::Error> {
        let config: CloudflareConfig = serde_json::from_value(provider_config.clone())?;

        let ids = self
            .list_record_ids(&config.token, &config.zone_id, record_uuid)
            .await?;

        if ids.is_empty() {
            return Ok(());
        }

        let deletes: Vec<serde_json::Value> = ids
            .into_iter()
            .map(|id| serde_json::json!({ "id": id }))
            .collect();

        self.batch(
            &config.token,
            &config.zone_id,
            serde_json::json!({ "deletes": deletes }),
        )
        .await
    }

    async fn test(&self, provider_config: &serde_json::Value) -> Result<bool, anyhow::Error> {
        let config: CloudflareConfig = serde_json::from_value(provider_config.clone())?;

        let verify_res = self
            .client
            .get("https://api.cloudflare.com/client/v4/user/tokens/verify")
            .bearer_auth(&config.token)
            .send()
            .await?;

        if !verify_res.status().is_success() {
            return Ok(false);
        }

        #[derive(Deserialize)]
        struct TokenVerifyResult {
            status: String,
        }
        #[derive(Deserialize)]
        struct TokenVerifyResponse {
            result: TokenVerifyResult,
        }
        let verify: TokenVerifyResponse = match verify_res.json().await {
            Ok(v) => v,
            Err(_) => return Ok(false),
        };

        if verify.result.status != "active" {
            return Ok(false);
        }

        let write_res = self
            .client
            .post(format!("{}/batch", Self::zone_url(&config.zone_id)))
            .bearer_auth(&config.token)
            .json(&serde_json::json!({}))
            .send()
            .await?;

        Ok(write_res.status().is_success())
    }
}
