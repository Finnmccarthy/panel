use super::State;
use utoipa_axum::{router::OpenApiRouter, routes};

mod get {
    use serde::{Deserialize, Serialize};
    use shared::{
        GetState,
        models::{server::GetServer, user::GetPermissionManager},
        response::{ApiResponse, ApiResponseResult},
    };
    use utoipa::ToSchema;

    #[derive(ToSchema, Deserialize, Serialize)]
    struct ResponseBuild {
        build: crate::mcjars::MinecraftBuild,
        latest: crate::mcjars::MinecraftBuild,
    }

    #[derive(ToSchema, Serialize)]
    struct Response {
        #[schema(inline)]
        build: Option<ResponseBuild>,
    }

    #[utoipa::path(get, path = "/", responses(
        (status = OK, body = inline(Response)),
    ), params(
        (
            "server" = uuid::Uuid,
            description = "The server ID",
            example = "123e4567-e89b-12d3-a456-426614174000",
        ),
    ))]
    pub async fn route(
        state: GetState,
        permissions: GetPermissionManager,
        server: GetServer,
    ) -> ApiResponseResult {
        permissions.has_server_permission("files.read")?;

        let build = state
            .cache
            .cached(
                &format!("server::{}::minecraft::version", server.uuid),
                5,
                || async {
                    let jar_hash = server
                        .node
                        .fetch_cached(&state.database)
                        .await?
                        .api_client(&state.database)
                        .await?
                        .get_servers_server_version(
                            server.uuid,
                            &wings_api::servers_server_version::get::Query {
                                game: Some(wings_api::Game::MinecraftJava),
                                ..Default::default()
                            },
                        )
                        .await
                        .map_or_else(|_| None, |r| Some(r.hash));

                    Ok::<_, anyhow::Error>(match jar_hash {
                        Some(hash) => crate::mcjars::lookup_sha256(
                            crate::mcjars::ApiContext::from_settings(&state.settings).await?,
                            &state,
                            &hash,
                        )
                        .await
                        .map_or_else(
                            |_| None,
                            |[build, latest]| Some(ResponseBuild { build, latest }),
                        ),
                        None => None,
                    })
                },
            )
            .await?;

        ApiResponse::new_serialized(Response { build }).ok()
    }
}

pub fn router(state: &State) -> OpenApiRouter<State> {
    OpenApiRouter::new()
        .routes(routes!(get::route))
        .with_state(state.clone())
}
