use anyhow::Context;
use serde_json::Value;
use std::{collections::BTreeSet, path::Path};

pub const SHIPPED_MANIFEST: &str = ".shipped.json";

pub fn deep_merge(target: Value, source: Value) -> Value {
    match (target, source) {
        (Value::Object(mut result), Value::Object(source)) => {
            for (key, value) in source {
                let merged = match result.get(&key) {
                    Some(existing) if existing.is_object() && value.is_object() => {
                        deep_merge(existing.clone(), value)
                    }
                    _ => value,
                };
                result.insert(key, merged);
            }

            Value::Object(result)
        }
        (_, source) => source,
    }
}

#[derive(Debug)]
pub struct Classification {
    pub new_languages: Vec<String>,
    pub overrides: Vec<String>,
}

impl Classification {
    pub fn is_customized(&self) -> bool {
        !self.new_languages.is_empty() || !self.overrides.is_empty()
    }
}

pub fn read_shipped_manifest(translations: &Path) -> BTreeSet<String> {
    std::fs::read(translations.join(SHIPPED_MANIFEST))
        .ok()
        .and_then(|raw| serde_json::from_slice(&raw).ok())
        .unwrap_or_default()
}

pub fn seed_shipped(shipped: &Path, translations: &Path) -> anyhow::Result<BTreeSet<String>> {
    std::fs::create_dir_all(translations.join("overrides"))
        .with_context(|| format!("creating {}/overrides", translations.display()))?;

    let mut names = read_shipped_manifest(translations);

    if let Ok(entries) = std::fs::read_dir(shipped) {
        for entry in entries {
            let entry = entry?;
            let Some(name) = entry.file_name().to_str().map(str::to_string) else {
                continue;
            };
            if !name.ends_with(".json") || !entry.path().is_file() {
                continue;
            }

            std::fs::copy(entry.path(), translations.join(&name))
                .with_context(|| format!("seeding {name}"))?;
            names.insert(name);
        }
    }

    std::fs::write(
        translations.join(SHIPPED_MANIFEST),
        serde_json::to_vec_pretty(&names)?,
    )
    .with_context(|| format!("writing {SHIPPED_MANIFEST}"))?;

    Ok(names)
}

pub fn classify(
    translations: &Path,
    shipped_names: &BTreeSet<String>,
) -> anyhow::Result<Classification> {
    let mut new_languages = Vec::new();
    let mut overrides = Vec::new();

    if let Ok(entries) = std::fs::read_dir(translations) {
        for entry in entries {
            let entry = entry?;
            let Some(name) = entry.file_name().to_str().map(str::to_string) else {
                continue;
            };
            if name == SHIPPED_MANIFEST || !name.ends_with(".json") || !entry.path().is_file() {
                continue;
            }
            if !shipped_names.contains(&name) {
                new_languages.push(name);
            }
        }
    }

    if let Ok(entries) = std::fs::read_dir(translations.join("overrides")) {
        for entry in entries {
            let entry = entry?;
            let Some(name) = entry.file_name().to_str().map(str::to_string) else {
                continue;
            };
            if !name.ends_with(".json") || !entry.path().is_file() {
                continue;
            }
            overrides.push(name);
        }
    }

    new_languages.sort();
    overrides.sort();

    Ok(Classification {
        new_languages,
        overrides,
    })
}

pub fn stage(
    translations: &Path,
    shipped: &Path,
    shipped_names: &BTreeSet<String>,
    out: &Path,
) -> anyhow::Result<Vec<String>> {
    let _ = std::fs::remove_dir_all(out);
    std::fs::create_dir_all(out).with_context(|| format!("creating {}", out.display()))?;

    let classification = classify(translations, shipped_names)?;

    for name in &classification.new_languages {
        std::fs::copy(translations.join(name), out.join(name))
            .with_context(|| format!("staging new language {name}"))?;
    }

    let mut skipped = Vec::new();

    for name in &classification.overrides {
        let staged = out.join(name);
        let base_path = if staged.is_file() {
            staged.clone()
        } else if shipped.join(name).is_file() {
            shipped.join(name)
        } else if translations.join(name).is_file() {
            translations.join(name)
        } else {
            std::fs::copy(translations.join("overrides").join(name), &staged)
                .with_context(|| format!("staging baseless override {name}"))?;
            continue;
        };

        let override_path = translations.join("overrides").join(name);
        let base_raw = read_bytes(&base_path)?;
        let override_raw = read_bytes(&override_path)?;

        let (Ok(base), Ok(patch)) = (
            parse_value(&base_raw, &base_path),
            parse_value(&override_raw, &override_path),
        ) else {
            skipped.push(name.clone());
            continue;
        };

        let merged = deep_merge(base, patch);
        std::fs::write(&staged, serde_json::to_vec_pretty(&merged)?)
            .with_context(|| format!("writing merged {name}"))?;
    }

    Ok(skipped)
}

fn read_bytes(path: &Path) -> anyhow::Result<Vec<u8>> {
    std::fs::read(path).with_context(|| format!("reading {}", path.display()))
}

fn parse_value(raw: &[u8], path: &Path) -> anyhow::Result<Value> {
    serde_json::from_slice(raw).with_context(|| format!("parsing {}", path.display()))
}
