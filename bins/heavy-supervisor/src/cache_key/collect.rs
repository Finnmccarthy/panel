use crate::{
    cache_key::{FileDigest, KeyInputs},
    config::Config,
};
use anyhow::Context;
use std::{
    collections::BTreeSet,
    path::{Path, PathBuf},
};

pub fn collect_key_inputs(
    config: &Config,
    shipped_names: &BTreeSet<String>,
) -> anyhow::Result<KeyInputs> {
    let (panel_version, target) = stock_binary_identity(&config.stock_binary)?;
    let zips = list_extension_zips(&config.extensions_dir)?;

    build_key_inputs(config, shipped_names, &zips, panel_version, target)
}

pub fn build_key_inputs(
    config: &Config,
    shipped_names: &BTreeSet<String>,
    zips: &[PathBuf],
    panel_version: String,
    target: String,
) -> anyhow::Result<KeyInputs> {
    Ok(KeyInputs {
        panel_version,
        target,
        bin_name: config.bin_name.clone(),
        extensions: digest_extensions(zips)?,
        translations: digest_translations(&config.translations_dir, shipped_names)?,
    })
}

fn stock_binary_identity(stock_binary: &Path) -> anyhow::Result<(String, String)> {
    let output = std::process::Command::new(stock_binary)
        .arg("version")
        .output()
        .with_context(|| format!("running {} version", stock_binary.display()))?;

    if !output.status.success() {
        anyhow::bail!(
            "{} version exited with {}",
            stock_binary.display(),
            output.status
        );
    }

    let stdout = String::from_utf8(output.stdout)
        .with_context(|| format!("{} version printed non-utf8 output", stock_binary.display()))?;

    parse_panel_version_output(&stdout)
}

pub fn parse_panel_version_output(stdout: &str) -> anyhow::Result<(String, String)> {
    let first_line = stdout.lines().next().unwrap_or("");
    let rest = first_line
        .strip_prefix("github.com/calagopus/panel(backend) ")
        .with_context(|| format!("unexpected version output: {first_line:?}"))?;
    let (version, target) = rest
        .rsplit_once(" (")
        .with_context(|| format!("unexpected version output: {first_line:?}"))?;
    let target = target
        .strip_suffix(')')
        .with_context(|| format!("unexpected version output: {first_line:?}"))?;

    Ok((version.to_string(), target.to_string()))
}

pub fn list_extension_zips(extensions_dir: &Path) -> anyhow::Result<Vec<PathBuf>> {
    let mut zips = Vec::new();

    let entries = match std::fs::read_dir(extensions_dir) {
        Ok(entries) => entries,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(zips),
        Err(err) => {
            return Err(err).with_context(|| format!("reading {}", extensions_dir.display()));
        }
    };

    for entry in entries {
        let entry = entry?;
        let Some(name) = entry.file_name().to_str().map(str::to_string) else {
            continue;
        };
        if !name.ends_with(".c7s.zip") || !entry.path().is_file() {
            continue;
        }

        zips.push(entry.path());
    }

    zips.sort();

    Ok(zips)
}

fn digest_extensions(zips: &[PathBuf]) -> anyhow::Result<Vec<FileDigest>> {
    let mut out = Vec::new();

    for zip in zips {
        let name = zip
            .file_name()
            .and_then(|name| name.to_str())
            .context("an extension archive lost its name between listing and digesting")?
            .to_string();

        out.push(FileDigest {
            sha256: super::digest_file(zip).with_context(|| format!("digesting {name}"))?,
            name,
        });
    }

    Ok(out)
}

fn digest_translations(
    translations_dir: &Path,
    shipped_names: &BTreeSet<String>,
) -> anyhow::Result<Vec<FileDigest>> {
    let classification = crate::translations::classify(translations_dir, shipped_names)?;
    if !classification.is_customized() {
        return Ok(Vec::new());
    }

    let mut out = Vec::new();

    for name in &classification.new_languages {
        out.push(FileDigest {
            sha256: super::digest_file(&translations_dir.join(name))
                .with_context(|| format!("digesting {name}"))?,
            name: name.clone(),
        });
    }

    for name in &classification.overrides {
        let relpath = format!("overrides/{name}");
        out.push(FileDigest {
            sha256: super::digest_file(&translations_dir.join("overrides").join(name))
                .with_context(|| format!("digesting {relpath}"))?,
            name: relpath,
        });
    }

    Ok(out)
}
