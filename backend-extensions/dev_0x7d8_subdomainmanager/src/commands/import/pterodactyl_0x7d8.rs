use anyhow::Context;
use clap::{Args, FromArgMatches};
use colored::Colorize;
use sqlx::Row;
use std::collections::HashMap;

#[derive(Args)]
pub struct Pterodactyl0x7d8Args {
    #[arg(
        short = 'e',
        long = "environment",
        help = "the environment variable file location for the pterodactyl panel",
        default_value = "/var/www/pterodactyl/.env",
        value_hint = clap::ValueHint::FilePath
    )]
    environment: String,
    #[arg(
        short = 't',
        long = "cloudflare-token",
        help = "the global Cloudflare API token from the old 0x7d8 installation",
        default_value = ""
    )]
    cloudflare_token: String,
}

pub struct Pterodactyl0x7d8Command;

impl shared::extensions::commands::CliCommand<Pterodactyl0x7d8Args> for Pterodactyl0x7d8Command {
    fn get_command(&self, command: clap::Command) -> clap::Command {
        command
    }

    fn get_executor(self) -> Box<shared::extensions::commands::ExecutorFunc> {
        Box::new(|env, arg_matches| {
            Box::pin(async move {
                let args = Pterodactyl0x7d8Args::from_arg_matches(&arg_matches)?;

                let start_time = std::time::Instant::now();
                let state = shared::AppState::new_cli(env).await?;

                if let Err(err) = dotenvy::from_path(&args.environment) {
                    eprintln!(
                        "{}: {:#?}",
                        "failed to read pterodactyl environment file".red(),
                        err
                    );

                    return Ok(1);
                }

                let source_database_host = match std::env::var("DB_HOST") {
                    Ok(value) => value,
                    Err(err) => {
                        eprintln!(
                            "{}: {:#?}",
                            "failed to read pterodactyl environment DB_HOST".red(),
                            err
                        );

                        return Ok(1);
                    }
                };
                let source_database_port = match std::env::var("DB_PORT").map(|v| v.parse::<u16>())
                {
                    Ok(Ok(value)) => value,
                    Ok(Err(err)) => {
                        eprintln!(
                            "{}: {:#?}",
                            "failed to read pterodactyl environment DB_PORT".red(),
                            err
                        );

                        return Ok(1);
                    }
                    Err(err) => {
                        eprintln!(
                            "{}: {:#?}",
                            "failed to read pterodactyl environment DB_PORT".red(),
                            err
                        );

                        return Ok(1);
                    }
                };
                let source_database_database = match std::env::var("DB_DATABASE") {
                    Ok(value) => value,
                    Err(err) => {
                        eprintln!(
                            "{}: {:#?}",
                            "failed to read pterodactyl environment DB_DATABASE".red(),
                            err
                        );

                        return Ok(1);
                    }
                };
                let source_database_username = match std::env::var("DB_USERNAME") {
                    Ok(value) => value,
                    Err(err) => {
                        eprintln!(
                            "{}: {:#?}",
                            "failed to read pterodactyl environment DB_USERNAME".red(),
                            err
                        );

                        return Ok(1);
                    }
                };
                let source_database_password = match std::env::var("DB_PASSWORD") {
                    Ok(value) => value,
                    Err(err) => {
                        eprintln!(
                            "{}: {:#?}",
                            "failed to read pterodactyl environment DB_PASSWORD".red(),
                            err
                        );

                        return Ok(1);
                    }
                };

                let source_connect_opts = sqlx::mysql::MySqlConnectOptions::new()
                    .host(source_database_host.trim_matches('"'))
                    .port(source_database_port)
                    .database(source_database_database.trim_matches('"'))
                    .username(source_database_username.trim_matches('"'))
                    .password(source_database_password.trim_matches('"'));
                let source_database = match sqlx::mysql::MySqlPoolOptions::new()
                    .connect_with(source_connect_opts)
                    .await
                {
                    Ok(database) => database,
                    Err(err) => {
                        eprintln!(
                            "{}: {:#?}",
                            "failed to connect to pterodactyl database".red(),
                            err
                        );

                        return Ok(1);
                    }
                };

                let egg_rows = sqlx::query("SELECT eggs.id, eggs.uuid FROM eggs")
                    .fetch_all(&source_database)
                    .await
                    .context("failed to fetch eggs from pterodactyl database")?;

                let mut egg_id_to_uuid: HashMap<u32, uuid::Uuid> =
                    HashMap::with_capacity(egg_rows.len());
                for row in &egg_rows {
                    let id: u32 = row.try_get("id")?;
                    let uuid: uuid::fmt::Hyphenated = row.try_get("uuid")?;
                    egg_id_to_uuid.insert(id, *uuid.as_uuid());
                }
                tracing::info!("fetched {} eggs", egg_id_to_uuid.len());

                let server_rows = sqlx::query("SELECT servers.id, servers.uuid FROM servers")
                    .fetch_all(&source_database)
                    .await
                    .context("failed to fetch servers from pterodactyl database")?;

                let mut server_id_to_uuid: HashMap<u32, uuid::Uuid> =
                    HashMap::with_capacity(server_rows.len());
                for row in &server_rows {
                    let id: u32 = row.try_get("id")?;
                    let uuid: uuid::fmt::Hyphenated = row.try_get("uuid")?;
                    server_id_to_uuid.insert(id, *uuid.as_uuid());
                }
                tracing::info!("fetched {} servers", server_id_to_uuid.len());

                let node_rows = sqlx::query("SELECT nodes.id, nodes.uuid FROM nodes")
                    .fetch_all(&source_database)
                    .await
                    .context("failed to fetch nodes from pterodactyl database")?;

                let mut node_id_to_uuid: HashMap<u32, uuid::Uuid> =
                    HashMap::with_capacity(node_rows.len());
                for row in &node_rows {
                    let id: u32 = row.try_get("id")?;
                    let uuid: uuid::fmt::Hyphenated = row.try_get("uuid")?;
                    node_id_to_uuid.insert(id, *uuid.as_uuid());
                }

                struct AllocationInfo {
                    node_uuid: uuid::Uuid,
                    ip: String,
                    port: u16,
                }

                let allocation_rows = sqlx::query(
                    "SELECT allocations.id, allocations.node_id, allocations.ip, allocations.port
                    FROM allocations",
                )
                .fetch_all(&source_database)
                .await
                .context("failed to fetch allocations from pterodactyl database")?;

                let mut allocation_id_to_info: HashMap<u32, AllocationInfo> =
                    HashMap::with_capacity(allocation_rows.len());
                for row in &allocation_rows {
                    let id: u32 = row.try_get("id")?;
                    let node_id: u32 = row.try_get("node_id")?;
                    let ip: String = row.try_get("ip")?;
                    let port: u16 = row.try_get("port")?;

                    let Some(&node_uuid) = node_id_to_uuid.get(&node_id) else {
                        continue;
                    };
                    allocation_id_to_info.insert(
                        id,
                        AllocationInfo {
                            node_uuid,
                            ip,
                            port,
                        },
                    );
                }
                tracing::info!("fetched {} allocations", allocation_id_to_info.len());

                drop(node_id_to_uuid);

                let subdomain_rows = sqlx::query(
                    "SELECT subdomains.id, subdomains.eggs, subdomains.domain, subdomains.zone_id,
                    subdomains.disallowed_subdomains_regexes, subdomains.api_data
                    FROM subdomains",
                )
                .fetch_all(&source_database)
                .await
                .context("failed to fetch subdomains from pterodactyl database")?;

                tracing::info!("importing {} domains", subdomain_rows.len());

                let mut subdomain_id_to_uuid: HashMap<u32, uuid::Uuid> =
                    HashMap::with_capacity(subdomain_rows.len());
                let mut subdomain_id_to_zone_id: HashMap<u32, String> =
                    HashMap::with_capacity(subdomain_rows.len());

                for row in &subdomain_rows {
                    let id: u32 = row.try_get("id")?;
                    let eggs_json: serde_json::Value = row.try_get("eggs")?;
                    let domain: &str = row.try_get("domain")?;
                    let zone_id: &str = row.try_get("zone_id")?;
                    let disallowed_json: serde_json::Value =
                        row.try_get("disallowed_subdomains_regexes")?;
                    let api_data: serde_json::Value = row.try_get("api_data")?;

                    let egg_uuids: Vec<uuid::Uuid> = eggs_json
                        .as_array()
                        .unwrap_or(&vec![])
                        .iter()
                        .filter_map(|value| {
                            let egg_id: u32 = value.as_str()?.parse().ok()?;
                            egg_id_to_uuid.get(&egg_id).copied()
                        })
                        .collect();

                    let disallowed_regexes: Vec<String> = disallowed_json
                        .as_array()
                        .unwrap_or(&vec![])
                        .iter()
                        .filter_map(|value| value.as_str().map(|s| s.to_string()))
                        .collect();

                    let provider_config = serde_json::json!({
                        "token": args.cloudflare_token,
                        "zone_id": zone_id,
                        "api_data": api_data,
                    });

                    let domain_row = sqlx::query(
                        "INSERT INTO dev_0x7d8_subdomainmanager_domains
                            (provider, domain, eggs, disallowed_subdomains_regexes, provider_config)
                        VALUES ($1, $2, $3, $4, $5)
                        RETURNING uuid",
                    )
                    .bind(crate::providers::Provider::Cloudflare)
                    .bind(domain)
                    .bind(&egg_uuids)
                    .bind(&disallowed_regexes)
                    .bind(provider_config)
                    .fetch_one(state.database.write())
                    .await
                    .context("failed to insert domain")?;

                    let domain_uuid: uuid::Uuid = domain_row.try_get("uuid")?;
                    subdomain_id_to_uuid.insert(id, domain_uuid);
                    subdomain_id_to_zone_id.insert(id, zone_id.to_string());

                    tracing::info!("imported domain {} ({})", domain, domain_uuid);
                }

                drop(egg_id_to_uuid);

                if args.cloudflare_token.is_empty() {
                    tracing::warn!(
                        "no --cloudflare-token provided, DNS comments will not be migrated"
                    );
                }

                let http_client = reqwest::Client::new();

                let server_subdomain_rows = sqlx::query(
                    "SELECT server_subdomains.id, server_subdomains.subdomain,
                    server_subdomains.server_id, server_subdomains.subdomain_id,
                    server_subdomains.allocation_id, server_subdomains.created_at
                    FROM server_subdomains",
                )
                .fetch_all(&source_database)
                .await
                .context("failed to fetch server_subdomains from pterodactyl database")?;

                tracing::info!(
                    "importing {} server subdomains",
                    server_subdomain_rows.len()
                );

                let mut imported_count: u32 = 0;
                let mut skipped_count: u32 = 0;

                for row in &server_subdomain_rows {
                    let old_id: u32 = row.try_get("id")?;
                    let subdomain: &str = row.try_get("subdomain")?;
                    let server_id: Option<u32> = row.try_get("server_id")?;
                    let subdomain_id: u32 = row.try_get("subdomain_id")?;
                    let allocation_id: Option<u32> = row.try_get("allocation_id")?;
                    let created_at: chrono::DateTime<chrono::Utc> = row.try_get("created_at")?;

                    let Some(&domain_uuid) = subdomain_id_to_uuid.get(&subdomain_id) else {
                        tracing::warn!(
                            "server subdomain '{}' references unknown subdomain id {}, skipping",
                            subdomain,
                            subdomain_id
                        );
                        skipped_count += 1;
                        continue;
                    };

                    let server_uuid = server_id.and_then(|id| server_id_to_uuid.get(&id).copied());

                    let allocation_uuid = if let Some(allocation_id) = allocation_id {
                        if let Some(allocation_info) = allocation_id_to_info.get(&allocation_id) {
                            sqlx::query(
                                "SELECT server_allocations.uuid
                                FROM server_allocations
                                JOIN node_allocations ON node_allocations.uuid = server_allocations.allocation_uuid
                                WHERE node_allocations.node_uuid = $1
                                    AND host(node_allocations.ip) = $2
                                    AND node_allocations.port = $3",
                            )
                            .bind(allocation_info.node_uuid)
                            .bind(allocation_info.ip.as_str())
                            .bind(allocation_info.port as i32)
                            .fetch_optional(state.database.read())
                            .await
                            .context("failed to resolve allocation")?
                            .map(|allocation_row| {
                                allocation_row.try_get::<uuid::Uuid, _>("uuid")
                            })
                            .transpose()?
                        } else {
                            None
                        }
                    } else {
                        None
                    };

                    let inserted_row = sqlx::query(
                        "INSERT INTO dev_0x7d8_subdomainmanager_server_subdomains
                            (subdomain, server_uuid, domain_uuid, allocation_uuid, created)
                        VALUES ($1, $2, $3, $4, $5)
                        ON CONFLICT (subdomain, domain_uuid) DO NOTHING
                        RETURNING uuid",
                    )
                    .bind(subdomain)
                    .bind(server_uuid)
                    .bind(domain_uuid)
                    .bind(allocation_uuid)
                    .bind(created_at)
                    .fetch_optional(state.database.write())
                    .await
                    .context("failed to insert server subdomain")?;

                    let Some(uuid_row) = inserted_row else {
                        skipped_count += 1;
                        continue;
                    };

                    imported_count += 1;

                    if !args.cloudflare_token.is_empty() {
                        let new_uuid: uuid::Uuid = uuid_row.try_get("uuid")?;

                        if let Some(zone_id) = subdomain_id_to_zone_id.get(&subdomain_id) {
                            let old_comment = format!("subdomainmanager.subdomain_id={old_id}");
                            let new_comment =
                                format!("dev.0x7d8.subdomainmanager.record_uuid={new_uuid}");
                            let zone_url = format!(
                                "https://api.cloudflare.com/client/v4/zones/{zone_id}/dns_records"
                            );

                            match http_client
                                .get(&zone_url)
                                .bearer_auth(&args.cloudflare_token)
                                .query(&[("comment", &old_comment)])
                                .send()
                                .await
                            {
                                Ok(list_res) if list_res.status().is_success() => {
                                    #[derive(serde::Deserialize)]
                                    struct ListResponse {
                                        result: Vec<serde_json::Value>,
                                    }

                                    let body: ListResponse = list_res
                                        .json()
                                        .await
                                        .context("failed to parse DNS records list")?;

                                    let record_ids: Vec<String> = body
                                        .result
                                        .into_iter()
                                        .filter_map(|record| {
                                            record.get("id")?.as_str().map(|s| s.to_string())
                                        })
                                        .collect();

                                    if record_ids.is_empty() {
                                        tracing::debug!(
                                            "no DNS records found for subdomain '{}' with old comment, skipping DNS migration",
                                            subdomain
                                        );
                                    } else {
                                        let patches: Vec<serde_json::Value> = record_ids
                                            .into_iter()
                                            .map(|record_id| {
                                                serde_json::json!({
                                                    "id": record_id,
                                                    "comment": new_comment,
                                                })
                                            })
                                            .collect();

                                        match http_client
                                            .post(format!("{zone_url}/batch"))
                                            .bearer_auth(&args.cloudflare_token)
                                            .json(&serde_json::json!({ "patches": patches }))
                                            .send()
                                            .await
                                        {
                                            Ok(batch_res) if batch_res.status().is_success() => {
                                                tracing::info!(
                                                    "migrated DNS comments for '{}' ({} -> {})",
                                                    subdomain,
                                                    old_comment,
                                                    new_comment,
                                                );
                                            }
                                            Ok(batch_res) => {
                                                tracing::warn!(
                                                    "failed to patch DNS comments for '{}': {}",
                                                    subdomain,
                                                    batch_res.text().await.unwrap_or_default()
                                                );
                                            }
                                            Err(err) => {
                                                tracing::warn!(
                                                    "failed to patch DNS comments for '{}': {err}",
                                                    subdomain,
                                                );
                                            }
                                        }
                                    }
                                }
                                Ok(list_res) => {
                                    tracing::warn!(
                                        "failed to list DNS records for '{}': {}",
                                        subdomain,
                                        list_res.text().await.unwrap_or_default()
                                    );
                                }
                                Err(err) => {
                                    tracing::warn!(
                                        "failed to list DNS records for '{}': {err}",
                                        subdomain,
                                    );
                                }
                            }
                        }
                    }
                }

                tracing::info!(
                    "imported {} server subdomains, skipped {}",
                    imported_count,
                    skipped_count
                );
                tracing::info!(
                    "finished processing import, took {:.2} seconds",
                    start_time.elapsed().as_secs_f32()
                );

                Ok(0)
            })
        })
    }
}
