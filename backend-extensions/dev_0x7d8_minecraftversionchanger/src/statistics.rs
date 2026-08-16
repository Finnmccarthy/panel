use indexmap::IndexMap;
use sqlx::Row;

pub async fn insert_installation(
    database: &shared::database::Database,
    server_uuid: uuid::Uuid,
    build: &crate::mcjars::MinecraftBuild,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO dev_0x7d8_minecraftversionchanger_installations (server_uuid, type_, version_id, project_version_id, build)
        VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(server_uuid)
    .bind(&build.r#type)
    .bind(&build.version_id)
    .bind(&build.project_version_id)
    .bind(&build.name)
    .execute(database.write())
    .await?;

    Ok(())
}

pub async fn get_total_installations(
    database: &shared::database::Database,
) -> Result<i64, sqlx::Error> {
    let count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM dev_0x7d8_minecraftversionchanger_installations")
            .fetch_one(database.read())
            .await?;

    Ok(count)
}

pub async fn get_installations_by_type(
    database: &shared::database::Database,
) -> Result<IndexMap<compact_str::CompactString, i64>, sqlx::Error> {
    let rows = sqlx::query(
        "SELECT type_, COUNT(*) as count
        FROM dev_0x7d8_minecraftversionchanger_installations
        GROUP BY type_
        ORDER BY count DESC",
    )
    .fetch_all(database.read())
    .await?;

    let mut installations_by_type = IndexMap::new();
    for row in rows {
        installations_by_type.insert(row.try_get("type_")?, row.try_get("count")?);
    }

    Ok(installations_by_type)
}

pub async fn get_installations_by_version(
    database: &shared::database::Database,
) -> Result<IndexMap<compact_str::CompactString, i64>, sqlx::Error> {
    let rows = sqlx::query(
        "SELECT version_id, COUNT(*) as count
        FROM dev_0x7d8_minecraftversionchanger_installations
        WHERE version_id IS NOT NULL
        GROUP BY version_id
        ORDER BY count DESC
        LIMIT 100",
    )
    .fetch_all(database.read())
    .await?;

    let mut installations_by_version = IndexMap::new();
    for row in rows {
        installations_by_version.insert(row.try_get("version_id")?, row.try_get("count")?);
    }

    Ok(installations_by_version)
}
