use crate::store::record::ExtensionRef;
use anyhow::Context;
use serde::Deserialize;

const APPLIED_STATUS: &str = "applied";

#[derive(Deserialize)]
struct ListedExtension {
    status: String,
    metadata_toml: ListedMetadata,
    cargo_toml: ListedCargo,
}

#[derive(Deserialize)]
struct ListedMetadata {
    package_name: String,
}

#[derive(Deserialize)]
struct ListedCargo {
    package: ListedCargoPackage,
}

#[derive(Deserialize)]
struct ListedCargoPackage {
    version: String,
}

#[derive(Debug)]
pub struct Verification {
    pub verified: bool,
    pub compiled_in: Vec<ExtensionRef>,
    pub failure_reason: Option<String>,
}

struct ParsedArray {
    start: usize,
    end: usize,
    entries: usize,
    listed: Listed,
}

#[derive(Clone, PartialEq)]
struct Listed {
    installed: Vec<ExtensionRef>,
    compiled_in: Vec<ExtensionRef>,
}

fn parse_listed(stdout: &str) -> anyhow::Result<Listed> {
    let mut parsed = Vec::new();
    let mut last_error = None;

    for (start, _) in stdout.match_indices('[') {
        match read_extensions(stdout, start) {
            Ok(array) => parsed.push(array),
            Err(err) => last_error = Some(err),
        }
    }

    let mut rivals = parsed
        .iter()
        .filter(|array| !is_nested_empty(array, &parsed));

    let Some(payload) = rivals.next() else {
        return match last_error {
            Some(err) => Err(err.context("parsing extensions list json")),
            None => Err(anyhow::anyhow!(
                "unexpected extensions list output: {:?}",
                stdout.lines().next().unwrap_or("")
            )),
        };
    };

    if let Some(rival) = rivals.find(|array| array.listed != payload.listed) {
        anyhow::bail!(
            "extensions list output holds disagreeing json arrays at offsets {} and {}",
            payload.start,
            rival.start
        );
    }

    Ok(payload.listed.clone())
}

pub fn parse_compiled_in(stdout: &str) -> anyhow::Result<Vec<ExtensionRef>> {
    Ok(parse_listed(stdout)?.compiled_in)
}

pub fn parse_installed(stdout: &str) -> anyhow::Result<Vec<ExtensionRef>> {
    Ok(parse_listed(stdout)?.installed)
}

fn is_nested_empty(array: &ParsedArray, parsed: &[ParsedArray]) -> bool {
    array.entries == 0
        && parsed
            .iter()
            .any(|other| other.start < array.start && array.end <= other.end)
}

fn read_extensions(stdout: &str, start: usize) -> anyhow::Result<ParsedArray> {
    let mut values =
        serde_json::Deserializer::from_str(&stdout[start..]).into_iter::<Vec<ListedExtension>>();
    let listed = values
        .next()
        .context("extensions list printed no json value")??;

    let mut installed = Vec::with_capacity(listed.len());
    let mut compiled_in = Vec::new();

    for extension in &listed {
        let reference = ExtensionRef {
            package_name: extension.metadata_toml.package_name.clone(),
            version: extension.cargo_toml.package.version.clone(),
        };

        if extension.status == APPLIED_STATUS {
            compiled_in.push(reference.clone());
        }
        installed.push(reference);
    }

    Ok(ParsedArray {
        start,
        end: start + values.byte_offset(),
        entries: listed.len(),
        listed: Listed {
            installed,
            compiled_in,
        },
    })
}

pub fn verify_output(stdout: &str, intended: &[ExtensionRef]) -> Verification {
    match parse_compiled_in(stdout) {
        Ok(compiled_in) => Verification {
            verified: crate::store::record::extensions_satisfied(intended, &compiled_in),
            compiled_in,
            failure_reason: None,
        },
        Err(err) => not_verified(err),
    }
}

fn not_verified(err: anyhow::Error) -> Verification {
    Verification {
        verified: false,
        compiled_in: Vec::new(),
        failure_reason: Some(format!("{err:#}")),
    }
}
