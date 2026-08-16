use garde::Validate;
use serde::{Deserialize, Serialize};
use shared::{
    database::DatabaseError,
    models::{
        BaseModel, ByUuid, CreatableModel, CreateListenerList, DeletableModel, DeleteHandlerList,
        InsertQueryBuilder, IntoAdminApiObject, IntoApiObject, ModelExtensionData,
        ModelExtensionList, Pagination, UpdatableModel, UpdateHandlerList, UpdateQueryBuilder,
    },
    prelude::IteratorExt,
};
use sqlx::{Row, postgres::PgRow};
use std::{
    collections::BTreeMap,
    sync::{Arc, LazyLock},
};
use utoipa::ToSchema;

#[derive(Serialize, Deserialize, Clone)]
pub struct Domain {
    pub uuid: uuid::Uuid,
    pub provider: crate::providers::Provider,
    pub domain: compact_str::CompactString,
    pub eggs: Vec<uuid::Uuid>,
    pub disallowed_subdomains_regexes: Vec<compact_str::CompactString>,
    pub provider_config: serde_json::Value,

    extension_data: ModelExtensionData,
}

impl BaseModel for Domain {
    const NAME: &'static str = "dev_0x7d8_subdomainmanager_domain";

    fn get_extension_list() -> &'static ModelExtensionList {
        static EXTENSIONS: LazyLock<ModelExtensionList> =
            LazyLock::new(|| parking_lot::RwLock::new(Vec::new()));

        &EXTENSIONS
    }

    fn get_extension_data(&self) -> &ModelExtensionData {
        &self.extension_data
    }

    #[inline]
    fn base_columns(prefix: Option<&str>) -> BTreeMap<&'static str, compact_str::CompactString> {
        let prefix = prefix.unwrap_or_default();

        BTreeMap::from([
            (
                "dev_0x7d8_subdomainmanager_domains.uuid",
                compact_str::format_compact!("{prefix}uuid"),
            ),
            (
                "dev_0x7d8_subdomainmanager_domains.provider",
                compact_str::format_compact!("{prefix}provider"),
            ),
            (
                "dev_0x7d8_subdomainmanager_domains.domain",
                compact_str::format_compact!("{prefix}domain"),
            ),
            (
                "dev_0x7d8_subdomainmanager_domains.eggs",
                compact_str::format_compact!("{prefix}eggs"),
            ),
            (
                "dev_0x7d8_subdomainmanager_domains.disallowed_subdomains_regexes",
                compact_str::format_compact!("{prefix}disallowed_subdomains_regexes"),
            ),
            (
                "dev_0x7d8_subdomainmanager_domains.provider_config",
                compact_str::format_compact!("{prefix}provider_config"),
            ),
        ])
    }

    #[inline]
    fn map(prefix: Option<&str>, row: &PgRow) -> Result<Self, DatabaseError> {
        let prefix = prefix.unwrap_or_default();

        Ok(Self {
            uuid: row.try_get(compact_str::format_compact!("{prefix}uuid").as_str())?,
            provider: row.try_get(compact_str::format_compact!("{prefix}provider").as_str())?,
            domain: row.try_get(compact_str::format_compact!("{prefix}domain").as_str())?,
            eggs: row.try_get(compact_str::format_compact!("{prefix}eggs").as_str())?,
            disallowed_subdomains_regexes: row.try_get(
                compact_str::format_compact!("{prefix}disallowed_subdomains_regexes").as_str(),
            )?,
            provider_config: row
                .try_get(compact_str::format_compact!("{prefix}provider_config").as_str())?,
            extension_data: Self::map_extensions(prefix, row)?,
        })
    }
}

impl Domain {
    pub async fn all_with_pagination(
        database: &shared::database::Database,
        page: i64,
        per_page: i64,
        search: Option<&str>,
    ) -> Result<Pagination<Self>, DatabaseError> {
        let offset = (page - 1) * per_page;

        let rows = sqlx::query(sqlx::AssertSqlSafe(format!(
            "SELECT {}, COUNT(*) OVER() AS total_count
            FROM dev_0x7d8_subdomainmanager_domains
            WHERE $1 IS NULL OR dev_0x7d8_subdomainmanager_domains.domain ILIKE '%' || $1 || '%'
            ORDER BY dev_0x7d8_subdomainmanager_domains.domain
            LIMIT $2 OFFSET $3",
            Self::columns_sql(None)
        )))
        .bind(search)
        .bind(per_page)
        .bind(offset)
        .fetch_all(database.read())
        .await?;

        Ok(Pagination {
            total: rows
                .first()
                .map_or(Ok(0), |row| row.try_get("total_count"))?,
            per_page,
            page,
            data: rows
                .iter()
                .map(|row| Self::map(None, row))
                .try_collect_vec()?,
        })
    }

    pub async fn all_by_egg_uuid(
        database: &shared::database::Database,
        egg_uuid: uuid::Uuid,
    ) -> Result<Vec<Self>, DatabaseError> {
        let rows = sqlx::query(sqlx::AssertSqlSafe(format!(
            "SELECT {}
            FROM dev_0x7d8_subdomainmanager_domains
            WHERE $1 = ANY(dev_0x7d8_subdomainmanager_domains.eggs)
            ORDER BY dev_0x7d8_subdomainmanager_domains.domain",
            Self::columns_sql(None)
        )))
        .bind(egg_uuid)
        .fetch_all(database.read())
        .await?;

        rows.iter()
            .map(|row| Self::map(None, row))
            .try_collect_vec()
    }

    pub async fn by_uuid_egg_uuid(
        database: &shared::database::Database,
        uuid: uuid::Uuid,
        egg_uuid: uuid::Uuid,
    ) -> Result<Option<Self>, DatabaseError> {
        let row = sqlx::query(sqlx::AssertSqlSafe(format!(
            "SELECT {}
            FROM dev_0x7d8_subdomainmanager_domains
            WHERE dev_0x7d8_subdomainmanager_domains.uuid = $1
                AND $2 = ANY(dev_0x7d8_subdomainmanager_domains.eggs)",
            Self::columns_sql(None)
        )))
        .bind(uuid)
        .bind(egg_uuid)
        .fetch_optional(database.read())
        .await?;

        row.map(|row| Self::map(None, &row)).transpose()
    }

    pub fn censor_provider_config(&mut self, client: reqwest::Client) {
        self.provider
            .build(client)
            .censor_config(&mut self.provider_config);
    }
}

#[async_trait::async_trait]
impl ByUuid for Domain {
    async fn by_uuid(
        database: &shared::database::Database,
        uuid: uuid::Uuid,
    ) -> Result<Self, DatabaseError> {
        let row = sqlx::query(sqlx::AssertSqlSafe(format!(
            "SELECT {}
            FROM dev_0x7d8_subdomainmanager_domains
            WHERE dev_0x7d8_subdomainmanager_domains.uuid = $1",
            Self::columns_sql(None)
        )))
        .bind(uuid)
        .fetch_one(database.read())
        .await?;

        Self::map(None, &row)
    }

    async fn by_uuid_with_transaction(
        transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        uuid: uuid::Uuid,
    ) -> Result<Self, DatabaseError> {
        let row = sqlx::query(sqlx::AssertSqlSafe(format!(
            "SELECT {}
            FROM dev_0x7d8_subdomainmanager_domains
            WHERE dev_0x7d8_subdomainmanager_domains.uuid = $1",
            Self::columns_sql(None)
        )))
        .bind(uuid)
        .fetch_one(&mut **transaction)
        .await?;

        Self::map(None, &row)
    }
}

#[derive(ToSchema, Validate, Deserialize)]
pub struct CreateDomainOptions {
    #[garde(skip)]
    pub provider: crate::providers::Provider,
    #[garde(length(chars, min = 1, max = 255))]
    #[schema(min_length = 1, max_length = 255)]
    pub domain: compact_str::CompactString,
    #[garde(length(max = 100))]
    pub eggs: Vec<uuid::Uuid>,
    #[garde(length(max = 100))]
    pub disallowed_subdomains_regexes: Vec<compact_str::CompactString>,
    #[garde(skip)]
    pub provider_config: serde_json::Value,
}

#[async_trait::async_trait]
impl CreatableModel for Domain {
    type CreateOptions<'a> = CreateDomainOptions;
    type CreateResult = Self;

    fn get_create_handlers() -> &'static LazyLock<CreateListenerList<Self>> {
        static CREATE_HANDLERS: LazyLock<CreateListenerList<Domain>> =
            LazyLock::new(|| Arc::new(shared::models::ModelHandlerList::default()));

        &CREATE_HANDLERS
    }

    async fn create_with_transaction(
        state: &shared::State,
        mut options: Self::CreateOptions<'_>,
        transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    ) -> Result<Self::CreateResult, DatabaseError> {
        options.validate()?;

        if let Err(report) = options
            .provider
            .build(state.client.clone())
            .validate_config(&options.provider_config)
        {
            return Err(DatabaseError::Validation(report));
        }

        let mut query_builder = InsertQueryBuilder::new("dev_0x7d8_subdomainmanager_domains");

        Self::run_create_handlers(&mut options, &mut query_builder, state, transaction).await?;

        query_builder
            .set("provider", options.provider)
            .set("domain", &options.domain)
            .set("eggs", &options.eggs)
            .set(
                "disallowed_subdomains_regexes",
                &options.disallowed_subdomains_regexes,
            )
            .set("provider_config", &options.provider_config);

        let row = query_builder
            .returning(&Self::columns_sql(None))
            .fetch_one(&mut **transaction)
            .await?;
        let mut domain = Self::map(None, &row)?;

        Self::run_after_create_handlers(&mut domain, &options, state, transaction).await?;

        Ok(domain)
    }
}

#[derive(ToSchema, Serialize, Deserialize, Validate, Clone, Default)]
pub struct UpdateDomainOptions {
    #[garde(skip)]
    pub provider: Option<crate::providers::Provider>,
    #[garde(length(chars, min = 1, max = 255))]
    #[schema(min_length = 1, max_length = 255)]
    pub domain: Option<compact_str::CompactString>,
    #[garde(length(max = 100))]
    pub eggs: Option<Vec<uuid::Uuid>>,
    #[garde(length(max = 100))]
    pub disallowed_subdomains_regexes: Option<Vec<compact_str::CompactString>>,
    #[garde(skip)]
    pub provider_config: Option<serde_json::Value>,
}

#[async_trait::async_trait]
impl UpdatableModel for Domain {
    type UpdateOptions = UpdateDomainOptions;

    fn get_update_handlers() -> &'static LazyLock<UpdateHandlerList<Self>> {
        static UPDATE_HANDLERS: LazyLock<UpdateHandlerList<Domain>> =
            LazyLock::new(|| Arc::new(shared::models::ModelHandlerList::default()));

        &UPDATE_HANDLERS
    }

    async fn update_with_transaction(
        &mut self,
        state: &shared::State,
        mut options: Self::UpdateOptions,
        transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    ) -> Result<(), DatabaseError> {
        options.validate()?;

        if let (Some(provider), Some(provider_config)) =
            (&options.provider, &options.provider_config)
            && let Err(err) = provider
                .build(state.client.clone())
                .validate_config(provider_config)
        {
            return Err(DatabaseError::Validation(err));
        }

        let mut query_builder = UpdateQueryBuilder::new("dev_0x7d8_subdomainmanager_domains");

        self.run_update_handlers(&mut options, &mut query_builder, state, transaction)
            .await?;

        query_builder
            .set("provider", options.provider.as_ref())
            .set("domain", options.domain.as_ref())
            .set("eggs", options.eggs.as_ref())
            .set(
                "disallowed_subdomains_regexes",
                options.disallowed_subdomains_regexes.as_ref(),
            )
            .set("provider_config", options.provider_config.as_ref())
            .where_eq("uuid", self.uuid);

        query_builder.execute(&mut **transaction).await?;

        if let Some(provider) = options.provider {
            self.provider = provider;
        }
        if let Some(domain) = options.domain {
            self.domain = domain;
        }
        if let Some(eggs) = options.eggs {
            self.eggs = eggs;
        }
        if let Some(disallowed_subdomains_regexes) = options.disallowed_subdomains_regexes {
            self.disallowed_subdomains_regexes = disallowed_subdomains_regexes;
        }
        if let Some(provider_config) = options.provider_config {
            self.provider_config = provider_config;
        }

        self.run_after_update_handlers(state, transaction).await?;

        Ok(())
    }
}

#[async_trait::async_trait]
impl DeletableModel for Domain {
    type DeleteOptions = ();

    fn get_delete_handlers() -> &'static LazyLock<DeleteHandlerList<Self>> {
        static DELETE_HANDLERS: LazyLock<DeleteHandlerList<Domain>> =
            LazyLock::new(|| Arc::new(shared::models::ModelHandlerList::default()));

        &DELETE_HANDLERS
    }

    async fn delete_with_transaction(
        &self,
        state: &shared::State,
        options: Self::DeleteOptions,
        transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    ) -> Result<(), anyhow::Error> {
        self.run_delete_handlers(&options, state, transaction)
            .await?;

        sqlx::query(
            "DELETE FROM dev_0x7d8_subdomainmanager_domains
            WHERE dev_0x7d8_subdomainmanager_domains.uuid = $1",
        )
        .bind(self.uuid)
        .execute(&mut **transaction)
        .await?;

        self.run_after_delete_handlers(&options, state, transaction)
            .await?;

        Ok(())
    }
}

#[async_trait::async_trait]
impl IntoAdminApiObject for Domain {
    type AdminApiObject = AdminApiDomain;
    type ExtraArgs<'a> = ();

    async fn into_admin_api_object<'a>(
        self,
        _state: &shared::State,
        _args: Self::ExtraArgs<'a>,
    ) -> Result<Self::AdminApiObject, DatabaseError> {
        Ok(AdminApiDomain {
            uuid: self.uuid,
            provider: self.provider,
            domain: self.domain,
            eggs: self.eggs,
            disallowed_subdomains_regexes: self.disallowed_subdomains_regexes,
            provider_config: self.provider_config,
        })
    }
}

#[async_trait::async_trait]
impl IntoApiObject for Domain {
    type ApiObject = crate::models::domain::ApiDomain;
    type ExtraArgs<'a> = ();

    async fn into_api_object<'a>(
        self,
        _state: &shared::State,
        _args: Self::ExtraArgs<'a>,
    ) -> Result<Self::ApiObject, DatabaseError> {
        Ok(crate::models::domain::ApiDomain {
            uuid: self.uuid,
            domain: self.domain,
        })
    }
}

#[derive(ToSchema, Serialize)]
#[schema(title = "AdminDomain")]
pub struct AdminApiDomain {
    pub uuid: uuid::Uuid,
    pub provider: crate::providers::Provider,
    pub domain: compact_str::CompactString,
    pub eggs: Vec<uuid::Uuid>,
    pub disallowed_subdomains_regexes: Vec<compact_str::CompactString>,
    pub provider_config: serde_json::Value,
}

#[derive(ToSchema, Serialize)]
#[schema(title = "Domain")]
pub struct ApiDomain {
    pub uuid: uuid::Uuid,
    pub domain: compact_str::CompactString,
}
