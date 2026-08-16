use garde::Validate;
use serde::{Deserialize, Serialize};
use shared::{
    database::DatabaseError,
    models::{
        BaseModel, ByUuid, CreatableModel, CreateListenerList, DeletableModel, DeleteHandlerList,
        Fetchable, InsertQueryBuilder, IntoAdminApiObject, IntoApiObject, ModelExtensionData,
        ModelExtensionList, Pagination, UpdatableModel, UpdateHandlerList, UpdateQueryBuilder,
        server::Server, server_allocation::ServerAllocation,
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
pub struct ServerSubdomain {
    pub uuid: uuid::Uuid,
    pub server: Option<Fetchable<shared::models::server::Server>>,
    pub allocation_uuid: Option<uuid::Uuid>,
    pub subdomain: compact_str::CompactString,
    pub domain: super::domain::Domain,
    pub created: chrono::NaiveDateTime,

    extension_data: ModelExtensionData,
}

impl BaseModel for ServerSubdomain {
    const NAME: &'static str = "dev_0x7d8_subdomainmanager_server_subdomain";

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

        let mut columns = BTreeMap::from([
            (
                "dev_0x7d8_subdomainmanager_server_subdomains.uuid",
                compact_str::format_compact!("{prefix}uuid"),
            ),
            (
                "dev_0x7d8_subdomainmanager_server_subdomains.server_uuid",
                compact_str::format_compact!("{prefix}server_uuid"),
            ),
            (
                "dev_0x7d8_subdomainmanager_server_subdomains.allocation_uuid",
                compact_str::format_compact!("{prefix}allocation_uuid"),
            ),
            (
                "dev_0x7d8_subdomainmanager_server_subdomains.subdomain",
                compact_str::format_compact!("{prefix}subdomain"),
            ),
            (
                "dev_0x7d8_subdomainmanager_server_subdomains.created",
                compact_str::format_compact!("{prefix}created"),
            ),
        ]);

        columns.extend(super::domain::Domain::base_columns(Some("dm_")));
        columns
    }

    #[inline]
    fn map(prefix: Option<&str>, row: &PgRow) -> Result<Self, DatabaseError> {
        let prefix = prefix.unwrap_or_default();

        Ok(Self {
            uuid: row.try_get(compact_str::format_compact!("{prefix}uuid").as_str())?,
            server: shared::models::server::Server::get_fetchable_from_row(
                row,
                compact_str::format_compact!("{prefix}server_uuid"),
            ),
            allocation_uuid: row
                .try_get(compact_str::format_compact!("{prefix}allocation_uuid").as_str())?,
            subdomain: row.try_get(compact_str::format_compact!("{prefix}subdomain").as_str())?,
            domain: super::domain::Domain::map(Some("dm_"), row)?,
            created: row.try_get(compact_str::format_compact!("{prefix}created").as_str())?,
            extension_data: Self::map_extensions(prefix, row)?,
        })
    }
}

impl ServerSubdomain {
    pub async fn all_by_server_uuid_with_pagination(
        database: &shared::database::Database,
        server_uuid: uuid::Uuid,
        page: i64,
        per_page: i64,
        search: Option<&str>,
    ) -> Result<Pagination<Self>, DatabaseError> {
        let offset = (page - 1) * per_page;

        let rows = sqlx::query(sqlx::AssertSqlSafe(format!(
            "SELECT {}, COUNT(*) OVER() AS total_count
            FROM dev_0x7d8_subdomainmanager_server_subdomains
            JOIN dev_0x7d8_subdomainmanager_domains ON dev_0x7d8_subdomainmanager_domains.uuid = dev_0x7d8_subdomainmanager_server_subdomains.domain_uuid
            WHERE dev_0x7d8_subdomainmanager_server_subdomains.server_uuid = $1 AND ($2 IS NULL OR dev_0x7d8_subdomainmanager_server_subdomains.subdomain ILIKE '%' || $2 || '%')
            LIMIT $3 OFFSET $4",
            Self::columns_sql(None)
        )))
        .bind(server_uuid)
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

    pub async fn by_detached_with_pagination(
        database: &shared::database::Database,
        page: i64,
        per_page: i64,
    ) -> Result<Pagination<Self>, DatabaseError> {
        let offset = (page - 1) * per_page;

        let rows = sqlx::query(sqlx::AssertSqlSafe(format!(
            "SELECT {}, COUNT(*) OVER() AS total_count
            FROM dev_0x7d8_subdomainmanager_server_subdomains
            JOIN dev_0x7d8_subdomainmanager_domains ON dev_0x7d8_subdomainmanager_domains.uuid = dev_0x7d8_subdomainmanager_server_subdomains.domain_uuid
            WHERE dev_0x7d8_subdomainmanager_server_subdomains.server_uuid IS NULL
            LIMIT $1 OFFSET $2",
            Self::columns_sql(None)
        )))
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

    pub async fn all_by_domain_uuid_with_pagination(
        database: &shared::database::Database,
        domain_uuid: uuid::Uuid,
        page: i64,
        per_page: i64,
        search: Option<&str>,
    ) -> Result<Pagination<Self>, DatabaseError> {
        let offset = (page - 1) * per_page;

        let rows = sqlx::query(sqlx::AssertSqlSafe(format!(
            "SELECT {}, COUNT(*) OVER() AS total_count
            FROM dev_0x7d8_subdomainmanager_server_subdomains
            JOIN dev_0x7d8_subdomainmanager_domains ON dev_0x7d8_subdomainmanager_domains.uuid = dev_0x7d8_subdomainmanager_server_subdomains.domain_uuid
            WHERE dev_0x7d8_subdomainmanager_server_subdomains.domain_uuid = $1
                AND ($2 IS NULL OR dev_0x7d8_subdomainmanager_server_subdomains.subdomain ILIKE '%' || $2 || '%')
            LIMIT $3 OFFSET $4",
            Self::columns_sql(None)
        )))
        .bind(domain_uuid)
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

    pub async fn by_server_uuid_uuid(
        database: &shared::database::Database,
        server_uuid: uuid::Uuid,
        uuid: uuid::Uuid,
    ) -> Result<Option<Self>, DatabaseError> {
        let row = sqlx::query(sqlx::AssertSqlSafe(format!(
            "SELECT {}
            FROM dev_0x7d8_subdomainmanager_server_subdomains
            JOIN dev_0x7d8_subdomainmanager_domains ON dev_0x7d8_subdomainmanager_domains.uuid = dev_0x7d8_subdomainmanager_server_subdomains.domain_uuid
            WHERE dev_0x7d8_subdomainmanager_server_subdomains.server_uuid = $1
                AND dev_0x7d8_subdomainmanager_server_subdomains.uuid = $2",
            Self::columns_sql(None)
        )))
        .bind(server_uuid)
        .bind(uuid)
        .fetch_optional(database.read())
        .await?;

        row.map(|row| Self::map(None, &row)).transpose()
    }

    pub async fn count_by_server_uuid(
        database: &shared::database::Database,
        server_uuid: uuid::Uuid,
    ) -> Result<i64, sqlx::Error> {
        sqlx::query_scalar(
            "SELECT COUNT(*)
            FROM dev_0x7d8_subdomainmanager_server_subdomains
            WHERE dev_0x7d8_subdomainmanager_server_subdomains.server_uuid = $1",
        )
        .bind(server_uuid)
        .fetch_one(database.read())
        .await
    }
}

#[async_trait::async_trait]
impl ByUuid for ServerSubdomain {
    async fn by_uuid(
        database: &shared::database::Database,
        uuid: uuid::Uuid,
    ) -> Result<Self, DatabaseError> {
        let row = sqlx::query(sqlx::AssertSqlSafe(format!(
            "SELECT {}
            FROM dev_0x7d8_subdomainmanager_server_subdomains
            JOIN dev_0x7d8_subdomainmanager_domains ON dev_0x7d8_subdomainmanager_domains.uuid = dev_0x7d8_subdomainmanager_server_subdomains.domain_uuid
            WHERE dev_0x7d8_subdomainmanager_server_subdomains.uuid = $1",
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
            FROM dev_0x7d8_subdomainmanager_server_subdomains
            JOIN dev_0x7d8_subdomainmanager_domains ON dev_0x7d8_subdomainmanager_domains.uuid = dev_0x7d8_subdomainmanager_server_subdomains.domain_uuid
            WHERE dev_0x7d8_subdomainmanager_server_subdomains.uuid = $1",
            Self::columns_sql(None)
        )))
        .bind(uuid)
        .fetch_one(&mut **transaction)
        .await?;

        Self::map(None, &row)
    }
}

#[derive(Validate)]
pub struct CreateServerSubdomainOptions<'a> {
    #[garde(skip)]
    pub server: &'a Server,
    #[garde(skip)]
    pub domain: &'a super::domain::Domain,
    #[garde(skip)]
    pub allocation_uuid: uuid::Uuid,

    #[garde(length(chars, min = 3, max = 32))]
    pub subdomain: compact_str::CompactString,
}

#[async_trait::async_trait]
impl CreatableModel for ServerSubdomain {
    type CreateOptions<'a> = CreateServerSubdomainOptions<'a>;
    type CreateResult = Self;

    fn get_create_handlers() -> &'static LazyLock<CreateListenerList<Self>> {
        static CREATE_HANDLERS: LazyLock<CreateListenerList<ServerSubdomain>> =
            LazyLock::new(|| Arc::new(shared::models::ModelHandlerList::default()));

        &CREATE_HANDLERS
    }

    async fn create_with_transaction(
        state: &shared::State,
        mut options: Self::CreateOptions<'_>,
        transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    ) -> Result<Self::CreateResult, DatabaseError> {
        options.validate()?;

        for regex_str in &options.domain.disallowed_subdomains_regexes {
            if regex_str.is_empty() {
                continue;
            }
            if let Ok(re) = regex::Regex::new(regex_str.as_str())
                && re.is_match(options.subdomain.as_str())
            {
                return Err(DatabaseError::Any(anyhow::anyhow!("subdomain not allowed")));
            }
        }

        let Some(server_allocation) = ServerAllocation::by_server_uuid_uuid(
            &state.database,
            options.server.uuid,
            options.allocation_uuid,
        )
        .await?
        else {
            return Err(DatabaseError::Any(anyhow::anyhow!("allocation not found")));
        };

        let mut insert = InsertQueryBuilder::new("dev_0x7d8_subdomainmanager_server_subdomains");

        Self::run_create_handlers(&mut options, &mut insert, state, transaction).await?;

        insert
            .set("subdomain", options.subdomain.as_str())
            .set("server_uuid", options.server.uuid)
            .set("domain_uuid", options.domain.uuid)
            .set("allocation_uuid", server_allocation.uuid);

        let row = insert
            .returning("uuid")
            .fetch_one(&mut **transaction)
            .await?;
        let mut subdomain =
            Self::by_uuid_with_transaction(transaction, row.try_get("uuid")?).await?;

        options
            .domain
            .provider
            .build(state.client.clone())
            .create_records(
                &options.domain.provider_config,
                crate::providers::PlaceholderParams {
                    subdomain: &subdomain,
                    allocation: &server_allocation,
                },
            )
            .await?;

        Self::run_after_create_handlers(&mut subdomain, &options, state, transaction).await?;

        Ok(subdomain)
    }
}

#[derive(Default, ToSchema, Validate, Serialize, Deserialize)]
pub struct UpdateServerSubdomainOptions {
    #[garde(skip)]
    pub allocation_uuid: Option<uuid::Uuid>,
}

#[async_trait::async_trait]
impl UpdatableModel for ServerSubdomain {
    type UpdateOptions = UpdateServerSubdomainOptions;

    fn get_update_handlers() -> &'static LazyLock<UpdateHandlerList<Self>> {
        static UPDATE_HANDLERS: LazyLock<UpdateHandlerList<ServerSubdomain>> =
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

        if let Some(allocation_uuid) = options.allocation_uuid {
            let Some(server) = &self.server else {
                return Err(DatabaseError::Any(anyhow::anyhow!("server not found")));
            };

            let Some(server_allocation) = ServerAllocation::by_server_uuid_uuid(
                &state.database,
                server.uuid,
                allocation_uuid,
            )
            .await?
            else {
                return Err(DatabaseError::Any(anyhow::anyhow!("allocation not found")));
            };

            self.domain
                .provider
                .build(state.client.clone())
                .update_records(
                    &self.domain.provider_config,
                    crate::providers::PlaceholderParams {
                        subdomain: self,
                        allocation: &server_allocation,
                    },
                )
                .await
                .map_err(DatabaseError::Any)?;
        }

        let mut query_builder =
            UpdateQueryBuilder::new("dev_0x7d8_subdomainmanager_server_subdomains");

        self.run_update_handlers(&mut options, &mut query_builder, state, transaction)
            .await?;

        query_builder
            .set("allocation_uuid", options.allocation_uuid)
            .where_eq("uuid", self.uuid);

        query_builder.execute(&mut **transaction).await?;

        if let Some(allocation_uuid) = options.allocation_uuid {
            self.allocation_uuid = Some(allocation_uuid);
        }

        self.run_after_update_handlers(state, transaction).await?;

        Ok(())
    }
}

#[async_trait::async_trait]
impl DeletableModel for ServerSubdomain {
    type DeleteOptions = ();

    fn get_delete_handlers() -> &'static LazyLock<DeleteHandlerList<Self>> {
        static DELETE_HANDLERS: LazyLock<DeleteHandlerList<ServerSubdomain>> =
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
            "DELETE FROM dev_0x7d8_subdomainmanager_server_subdomains
            WHERE dev_0x7d8_subdomainmanager_server_subdomains.uuid = $1",
        )
        .bind(self.uuid)
        .execute(&mut **transaction)
        .await?;

        self.domain
            .provider
            .build(state.client.clone())
            .delete_records(&self.domain.provider_config, self.uuid)
            .await?;

        self.run_after_delete_handlers(&options, state, transaction)
            .await?;

        Ok(())
    }
}

#[async_trait::async_trait]
impl IntoAdminApiObject for ServerSubdomain {
    type AdminApiObject = AdminApiServerSubdomain;
    type ExtraArgs<'a> = &'a shared::storage::StorageUrlRetriever<'a>;

    async fn into_admin_api_object<'a>(
        self,
        state: &shared::State,
        storage_url_retriever: Self::ExtraArgs<'a>,
    ) -> Result<Self::AdminApiObject, DatabaseError> {
        let server = if let Some(server) = self.server {
            Some(server.fetch_cached(&state.database).await?)
        } else {
            None
        };

        let allocation = if let Some(allocation_uuid) = self.allocation_uuid
            && let Some(server) = &server
        {
            if let Some(server_allocation) =
                ServerAllocation::by_server_uuid_uuid(&state.database, server.uuid, allocation_uuid)
                    .await?
            {
                Some(
                    server_allocation
                        .into_api_object(state, server.allocation.as_ref().map(|a| a.uuid))
                        .await?,
                )
            } else {
                None
            }
        } else {
            None
        };
        let server = if let Some(server) = server {
            Some(
                server
                    .into_admin_api_object(state, storage_url_retriever)
                    .await?,
            )
        } else {
            None
        };

        Ok(AdminApiServerSubdomain {
            uuid: self.uuid,
            server,
            allocation,
            subdomain: self.subdomain,
            created: self.created.and_utc(),
        })
    }
}

#[async_trait::async_trait]
impl IntoApiObject for ServerSubdomain {
    type ApiObject = ApiServerSubdomain;
    type ExtraArgs<'a> = &'a shared::models::server::Server;

    async fn into_api_object<'a>(
        self,
        state: &shared::State,
        server: Self::ExtraArgs<'a>,
    ) -> Result<Self::ApiObject, DatabaseError> {
        let allocation = if let Some(alloc_uuid) = self.allocation_uuid {
            if let Some(server_allocation) =
                ServerAllocation::by_server_uuid_uuid(&state.database, server.uuid, alloc_uuid)
                    .await?
            {
                Some(
                    server_allocation
                        .into_api_object(state, server.allocation.as_ref().map(|a| a.uuid))
                        .await?,
                )
            } else {
                None
            }
        } else {
            None
        };

        Ok(ApiServerSubdomain {
            uuid: self.uuid,
            subdomain: self.subdomain,
            domain: self.domain.into_api_object(state, ()).await?,
            allocation,
            created: self.created.and_utc(),
        })
    }
}

#[derive(ToSchema, Serialize)]
#[schema(title = "AdminServerSubdomain")]
pub struct AdminApiServerSubdomain {
    pub uuid: uuid::Uuid,
    pub server: Option<shared::models::server::AdminApiServer>,
    pub allocation: Option<shared::models::server_allocation::ApiServerAllocation>,

    pub subdomain: compact_str::CompactString,

    pub created: chrono::DateTime<chrono::Utc>,
}

#[derive(ToSchema, Serialize)]
#[schema(title = "ServerSubdomain")]
pub struct ApiServerSubdomain {
    pub uuid: uuid::Uuid,
    pub domain: super::domain::ApiDomain,
    pub allocation: Option<shared::models::server_allocation::ApiServerAllocation>,

    pub subdomain: compact_str::CompactString,

    pub created: chrono::DateTime<chrono::Utc>,
}
