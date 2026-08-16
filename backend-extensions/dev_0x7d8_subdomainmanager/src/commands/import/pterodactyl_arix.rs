use anyhow::Context;
use clap::{Args, FromArgMatches};
use colored::Colorize;
use sqlx::Row;
use std::collections::HashMap;

#[derive(Args)]
pub struct PterodactylArixArgs {
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
        help = "a Cloudflare API token used for the imported domains and to migrate existing DNS record comments",
        default_value = ""
    )]
    cloudflare_token: String,
}

pub struct PterodactylArixCommand;

impl shared::extensions::commands::CliCommand<PterodactylArixArgs> for PterodactylArixCommand {
    fn get_command(&self, command: clap::Command) -> clap::Command {
        command
    }

    fn get_executor(self) -> Box<shared::extensions::commands::ExecutorFunc> {
        Box::new(|env, arg_matches| {
            Box::pin(async move {
                let args = PterodactylArixArgs::from_arg_matches(&arg_matches)?;

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

                if args.cloudflare_token.is_empty() {
                    tracing::warn!(
                        "no --cloudflare-token provided, imported domains will have no API token and existing DNS record comments will not be migrated"
                    );
                }

                let config_row = sqlx::query(
                    "SELECT subdomains_cloudflare_config.proxy_records,
                    subdomains_cloudflare_config.use_alias
                    FROM subdomains_cloudflare_config
                    LIMIT 1",
                )
                .fetch_optional(&source_database)
                .await
                .context("failed to fetch cloudflare config from arix database")?;

                let (proxy_records, use_alias) = match config_row {
                    Some(row) => {
                        let proxy_records: bool = row.try_get("proxy_records")?;
                        let use_alias: bool = row.try_get("use_alias")?;
                        (proxy_records, use_alias)
                    }
                    None => {
                        tracing::warn!(
                            "no subdomains_cloudflare_config row found, defaulting to no proxy and raw IP records"
                        );
                        (false, false)
                    }
                };

                let record_content = if use_alias {
                    "{ip_alias_forceip}"
                } else {
                    "{ip}"
                };
                let api_data = serde_json::json!({
                    "posts": [
                        {
                            "type": "A",
                            "name": "{subdomain}.{domain}",
                            "content": record_content,
                            "proxied": proxy_records,
                            "comment": "{comment}",
                            "ttl": 120,
                        },
                        {
                            "type": "SRV",
                            "name": "_minecraft._tcp.{subdomain}.{domain}",
                            "data": {
                                "priority": 0,
                                "weight": 5,
                                "port": "{port}",
                                "target": "{subdomain}.{domain}",
                            },
                            "comment": "{comment}",
                            "ttl": 120,
                        },
                    ],
                });

                let blocklist_rows =
                    sqlx::query("SELECT subdomains_blocklist.subdomain FROM subdomains_blocklist")
                        .fetch_all(&source_database)
                        .await
                        .context("failed to fetch blocklist from arix database")?;

                let mut disallowed_regexes: Vec<String> = Vec::with_capacity(blocklist_rows.len());
                for row in &blocklist_rows {
                    let subdomain: &str = row.try_get("subdomain")?;
                    disallowed_regexes.push(format!("^{}$", regex::escape(subdomain)));
                }
                tracing::info!(
                    "loaded {} blocklist entries as disallowed subdomain regexes",
                    disallowed_regexes.len()
                );

                let domain_rows = sqlx::query(
                    "SELECT subdomains_connected_domains.domain,
                    subdomains_connected_domains.zone_id
                    FROM subdomains_connected_domains",
                )
                .fetch_all(&source_database)
                .await
                .context("failed to fetch connected domains from arix database")?;

                tracing::info!("importing {} domains", domain_rows.len());

                struct DomainInfo {
                    uuid: uuid::Uuid,
                    zone_id: Option<String>,
                }

                let mut domain_to_info: HashMap<String, DomainInfo> =
                    HashMap::with_capacity(domain_rows.len());
                for row in &domain_rows {
                    let domain: &str = row.try_get("domain")?;
                    let zone_id: Option<String> = row.try_get("zone_id")?;

                    if zone_id.is_none() {
                        tracing::warn!(
                            "domain '{}' has no zone_id, importing without a working Cloudflare zone",
                            domain
                        );
                    }

                    let provider_config = serde_json::json!({
                        "token": args.cloudflare_token,
                        "zone_id": zone_id.clone().unwrap_or_default(),
                        "api_data": api_data,
                    });

                    let empty_eggs: Vec<uuid::Uuid> = Vec::new();
                    let domain_row = sqlx::query(
                        "INSERT INTO dev_0x7d8_subdomainmanager_domains
                            (provider, domain, eggs, disallowed_subdomains_regexes, provider_config)
                        VALUES ($1, $2, $3, $4, $5)
                        RETURNING uuid",
                    )
                    .bind(crate::providers::Provider::Cloudflare)
                    .bind(domain)
                    .bind(&empty_eggs)
                    .bind(&disallowed_regexes)
                    .bind(provider_config)
                    .fetch_one(state.database.write())
                    .await
                    .context("failed to insert domain")?;

                    let domain_uuid: uuid::Uuid = domain_row.try_get("uuid")?;
                    domain_to_info.insert(
                        domain.to_string(),
                        DomainInfo {
                            uuid: domain_uuid,
                            zone_id,
                        },
                    );

                    tracing::info!("imported domain {} ({})", domain, domain_uuid);
                }

                let server_rows =
                    sqlx::query("SELECT servers.id, servers.uuid, servers.uuidShort FROM servers")
                        .fetch_all(&source_database)
                        .await
                        .context("failed to fetch servers from pterodactyl database")?;

                let mut server_identifier_to_uuid: HashMap<String, uuid::Uuid> =
                    HashMap::with_capacity(server_rows.len());
                for row in &server_rows {
                    let id: u32 = row.try_get("id")?;
                    let uuid: uuid::fmt::Hyphenated = row.try_get("uuid")?;
                    let uuid_short: String = row.try_get("uuidShort")?;

                    server_identifier_to_uuid.insert(id.to_string(), *uuid.as_uuid());
                    server_identifier_to_uuid.insert(uuid_short, *uuid.as_uuid());
                    server_identifier_to_uuid.insert(uuid.to_string(), *uuid.as_uuid());
                }
                tracing::info!("fetched {} servers", server_rows.len());

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

                let user_subdomain_rows = sqlx::query(
                    "SELECT subdomains_user_subdomains.subdomain,
                    subdomains_user_subdomains.domain, subdomains_user_subdomains.full_domain,
                    subdomains_user_subdomains.server_id, subdomains_user_subdomains.port_id,
                    subdomains_user_subdomains.created_at
                    FROM subdomains_user_subdomains",
                )
                .fetch_all(&source_database)
                .await
                .context("failed to fetch user subdomains from arix database")?;

                tracing::info!("importing {} server subdomains", user_subdomain_rows.len());

                let http_client = reqwest::Client::new();

                let mut imported_count: u32 = 0;
                let mut skipped_count: u32 = 0;

                for row in &user_subdomain_rows {
                    let subdomain: &str = row.try_get("subdomain")?;
                    let domain: &str = row.try_get("domain")?;
                    let full_domain: &str = row.try_get("full_domain")?;
                    let server_id: &str = row.try_get("server_id")?;
                    let port_id: i32 = row.try_get("port_id")?;
                    let created_at: Option<chrono::DateTime<chrono::Utc>> =
                        row.try_get("created_at")?;

                    let Some(domain_info) = domain_to_info.get(domain) else {
                        tracing::warn!(
                            "server subdomain '{}' references unknown domain '{}', skipping",
                            subdomain,
                            domain
                        );
                        skipped_count += 1;
                        continue;
                    };

                    let server_uuid = server_identifier_to_uuid.get(server_id).copied();
                    if server_uuid.is_none() {
                        tracing::warn!(
                            "server subdomain '{}' references unknown server '{}', importing without a server link",
                            subdomain,
                            server_id
                        );
                    }

                    let allocation_uuid = if let Some(allocation_info) =
                        allocation_id_to_info.get(&(port_id as u32))
                    {
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
                            .map(|allocation_row| allocation_row.try_get::<uuid::Uuid, _>("uuid"))
                            .transpose()?
                    } else {
                        None
                    };

                    let created = created_at.unwrap_or_else(chrono::Utc::now);

                    let inserted_row = sqlx::query(
                        "INSERT INTO dev_0x7d8_subdomainmanager_server_subdomains
                            (subdomain, server_uuid, domain_uuid, allocation_uuid, created)
                        VALUES ($1, $2, $3, $4, $5)
                        ON CONFLICT (subdomain, domain_uuid) DO NOTHING
                        RETURNING uuid",
                    )
                    .bind(subdomain)
                    .bind(server_uuid)
                    .bind(domain_info.uuid)
                    .bind(allocation_uuid)
                    .bind(created)
                    .fetch_optional(state.database.write())
                    .await
                    .context("failed to insert server subdomain")?;

                    let Some(uuid_row) = inserted_row else {
                        skipped_count += 1;
                        continue;
                    };

                    imported_count += 1;

                    if args.cloudflare_token.is_empty() {
                        continue;
                    }

                    let Some(zone_id) = domain_info.zone_id.as_deref() else {
                        continue;
                    };

                    let new_uuid: uuid::Uuid = uuid_row.try_get("uuid")?;
                    let new_comment = format!("dev.0x7d8.subdomainmanager.record_uuid={new_uuid}");
                    let zone_url =
                        format!("https://api.cloudflare.com/client/v4/zones/{zone_id}/dns_records");

                    let record_names = [
                        full_domain.to_string(),
                        format!("_minecraft._tcp.{full_domain}"),
                    ];

                    let mut record_ids: Vec<String> = Vec::new();
                    for record_name in &record_names {
                        match http_client
                            .get(&zone_url)
                            .bearer_auth(&args.cloudflare_token)
                            .query(&[("name", record_name)])
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

                                record_ids.extend(body.result.into_iter().filter_map(|record| {
                                    record.get("id")?.as_str().map(|s| s.to_string())
                                }));
                            }
                            Ok(list_res) => {
                                tracing::warn!(
                                    "failed to list DNS records for '{}': {}",
                                    record_name,
                                    list_res.text().await.unwrap_or_default()
                                );
                            }
                            Err(err) => {
                                tracing::warn!(
                                    "failed to list DNS records for '{}': {err}",
                                    record_name,
                                );
                            }
                        }
                    }

                    if record_ids.is_empty() {
                        tracing::debug!(
                            "no DNS records found for '{}', skipping comment migration",
                            full_domain
                        );
                        continue;
                    }

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
                                "migrated DNS comments for '{}' -> {}",
                                full_domain,
                                new_comment,
                            );
                        }
                        Ok(batch_res) => {
                            tracing::warn!(
                                "failed to patch DNS comments for '{}': {}",
                                full_domain,
                                batch_res.text().await.unwrap_or_default()
                            );
                        }
                        Err(err) => {
                            tracing::warn!(
                                "failed to patch DNS comments for '{}': {err}",
                                full_domain,
                            );
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
