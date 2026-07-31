use sha2::Digest;
use std::path::Path;

pub mod collect;

pub struct FileDigest {
    pub name: String,
    pub sha256: String,
}

pub struct KeyInputs {
    pub panel_version: String,
    pub target: String,
    pub bin_name: String,
    pub extensions: Vec<FileDigest>,
    pub translations: Vec<FileDigest>,
}

fn escape_manifest_name(name: &str) -> String {
    name.chars()
        .flat_map(|c| match c {
            '\\' => vec!['\\', '\\'],
            '\n' => vec!['\\', 'n'],
            '\r' => vec!['\\', 'r'],
            ' ' => vec!['\\', 's'],
            other => vec![other],
        })
        .collect()
}

fn sort_key(entry: &FileDigest) -> (&[u8], &[u8]) {
    (entry.name.as_bytes(), entry.sha256.as_bytes())
}

pub fn canonical_manifest(inputs: &KeyInputs) -> String {
    let mut extensions: Vec<&FileDigest> = inputs.extensions.iter().collect();
    let mut translations: Vec<&FileDigest> = inputs.translations.iter().collect();
    extensions.sort_by_key(|entry| sort_key(entry));
    translations.sort_by_key(|entry| sort_key(entry));

    let mut manifest = String::new();
    manifest.push_str(&format!(
        "panel {}\n",
        escape_manifest_name(&inputs.panel_version)
    ));
    manifest.push_str(&format!(
        "target {}\n",
        escape_manifest_name(&inputs.target)
    ));
    manifest.push_str(&format!("bin {}\n", escape_manifest_name(&inputs.bin_name)));

    for entry in extensions {
        manifest.push_str(&format!(
            "ext {} {}\n",
            escape_manifest_name(&entry.name),
            entry.sha256
        ));
    }
    for entry in translations {
        manifest.push_str(&format!(
            "tr {} {}\n",
            escape_manifest_name(&entry.name),
            entry.sha256
        ));
    }

    manifest
}

pub fn cache_key(inputs: &KeyInputs) -> String {
    hex::encode(sha2::Sha256::digest(canonical_manifest(inputs).as_bytes()))
}

pub fn sanitize_version(version: &str) -> String {
    let sanitized: String = version
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '.' || c == '-' {
                c
            } else if c == ':' || c == '@' {
                '-'
            } else {
                '_'
            }
        })
        .collect();

    match sanitized.as_str() {
        "" | "." | ".." | ".state" => "unknown".to_string(),
        _ => sanitized,
    }
}

pub fn digest_file(path: &Path) -> std::io::Result<String> {
    let bytes = std::fs::read(path)?;

    Ok(hex::encode(sha2::Sha256::digest(&bytes)))
}
