use std::{
    path::{Path, PathBuf},
    time::Duration,
};

pub const DOCS_URL: &str =
    "https://calagopus.com/docs/panel/extensions/switching-to-the-heavy-image";

const DEFAULT_BIN_NAME: &str = "panel-rs";
const DEFAULT_SHUTDOWN_GRACE_SECS: u64 = 30;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuildProfile {
    Dev,
    Balanced,
    Optimized,
}

impl BuildProfile {
    fn parse(raw: &str) -> Option<Self> {
        match raw {
            "dev" => Some(Self::Dev),
            "balanced" => Some(Self::Balanced),
            "optimized" => Some(Self::Optimized),
            _ => None,
        }
    }

    pub fn as_arg(self) -> &'static str {
        match self {
            Self::Dev => "dev",
            Self::Balanced => "balanced",
            Self::Optimized => "optimized",
        }
    }

    pub fn target_path(self) -> &'static str {
        match self {
            Self::Dev => "debug",
            Self::Balanced => "heavy-release",
            Self::Optimized => "release",
        }
    }
}

pub struct Config {
    pub bin_name: String,
    pub repo_dir: PathBuf,
    pub binaries_dir: PathBuf,
    pub extensions_dir: PathBuf,
    pub translations_dir: PathBuf,
    pub shipped_translations_dir: PathBuf,
    pub staged_translations_dir: PathBuf,
    pub stock_binary: PathBuf,
    pub target_binary: PathBuf,
    pub socket_path: PathBuf,
    pub profile: BuildProfile,
    pub shutdown_grace: Duration,
    pub panel: crate::panel::Policy,
}

fn panel_policy(env: &dyn Fn(&str) -> Option<String>) -> crate::panel::Policy {
    let millis = |key: &str| {
        env(key)
            .and_then(|raw| raw.parse::<u64>().ok())
            .filter(|value| *value > 0)
            .map(Duration::from_millis)
    };

    let mut policy = crate::panel::Policy::default();

    if let Some(probe) = millis("SUPERVISOR_STARTUP_PROBE_MS") {
        policy.startup_probe = probe;
    }
    if let Some(backoff) = millis("SUPERVISOR_BACKOFF_BASE_MS") {
        policy.backoff_base = backoff;
    }

    policy
}

pub fn debug_logging(env: &dyn Fn(&str) -> Option<String>) -> bool {
    env("APP_DEBUG").is_some_and(|raw| raw.trim_matches('"').parse().unwrap_or(false))
}

#[derive(Debug)]
pub struct PreflightError {
    pub missing: PathBuf,
}

impl std::fmt::Display for PreflightError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} is missing or is not a directory, mount it as a volume. read {}",
            self.missing.display(),
            DOCS_URL
        )
    }
}

impl std::error::Error for PreflightError {}

impl Config {
    pub fn resolve(root: &Path, env: &dyn Fn(&str) -> Option<String>) -> Self {
        let bin_name = env("PANEL_BIN_NAME")
            .filter(|name| name == "panel-rs" || name == "panel-rs-aio")
            .unwrap_or_else(|| DEFAULT_BIN_NAME.to_string());
        let profile = match env("CARGO_BUILD_PROFILE") {
            Some(raw) => BuildProfile::parse(&raw).unwrap_or_else(|| {
                tracing::warn!("CARGO_BUILD_PROFILE={raw} is not a profile `extensions apply` accepts, building `balanced`");

                BuildProfile::Balanced
            }),
            None => BuildProfile::Balanced,
        };
        let shutdown_grace = env("SUPERVISOR_SHUTDOWN_GRACE")
            .and_then(|raw| raw.parse::<u64>().ok())
            .unwrap_or(DEFAULT_SHUTDOWN_GRACE_SECS);

        let app_dir = root.join("app");
        let repo_dir = app_dir.join("repo");

        Self {
            stock_binary: app_dir.join(format!("{bin_name}.stock")),
            target_binary: repo_dir
                .join("target")
                .join(profile.target_path())
                .join(&bin_name),
            binaries_dir: app_dir.join("binaries"),
            extensions_dir: app_dir.join("extensions"),
            translations_dir: app_dir.join("translations"),
            shipped_translations_dir: repo_dir.join("frontend/public/translations"),
            staged_translations_dir: root.join("tmp").join("staged-translations"),
            socket_path: root.join("tmp").join("calagopus").join("supervisor.sock"),
            repo_dir,
            bin_name,
            profile,
            shutdown_grace: Duration::from_secs(shutdown_grace),
            panel: panel_policy(env),
        }
    }

    pub fn preflight(&self) -> Result<(), PreflightError> {
        let migrations = self.repo_dir.join("database/extension-migrations");
        let required: [&Path; 4] = [
            &self.binaries_dir,
            &self.translations_dir,
            &self.extensions_dir,
            &migrations,
        ];

        for path in required {
            if !path.is_dir() {
                return Err(PreflightError {
                    missing: path.to_path_buf(),
                });
            }
        }

        Ok(())
    }
}
