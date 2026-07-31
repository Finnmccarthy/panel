use std::{path::PathBuf, time::SystemTime};

#[derive(Debug, Clone)]
pub struct EntryView {
    pub path: PathBuf,
    pub version: String,
    pub key: String,
    pub bin_name: String,
    pub verified: bool,
    pub modified: SystemTime,
}

pub struct BootInputs<'a> {
    pub key: &'a str,
    pub version: &'a str,
    pub bin_name: &'a str,
    pub has_extensions: bool,
    pub translations_customized: bool,
    pub failure_recorded: bool,
    pub entries: &'a [EntryView],
}

#[derive(Debug)]
pub enum BootDecision {
    Exact { entry: PathBuf },
    Stock,
    FallbackSuppressed { entry: Option<PathBuf> },
    FallbackAndBuild { entry: Option<PathBuf> },
}

pub fn exact_match<'a>(
    entries: &'a [EntryView],
    key: &str,
    bin_name: &str,
) -> Option<&'a EntryView> {
    entries
        .iter()
        .find(|entry| entry.key == key && entry.bin_name == bin_name && entry.verified)
}

pub fn decide(inputs: &BootInputs<'_>) -> BootDecision {
    if let Some(exact) = exact_match(inputs.entries, inputs.key, inputs.bin_name) {
        return BootDecision::Exact {
            entry: exact.path.clone(),
        };
    }

    if !inputs.has_extensions && !inputs.translations_customized {
        return BootDecision::Stock;
    }

    let fallback = fallback(inputs.entries, inputs.version, inputs.bin_name);

    if inputs.failure_recorded {
        return BootDecision::FallbackSuppressed { entry: fallback };
    }

    BootDecision::FallbackAndBuild { entry: fallback }
}

pub fn fallback(entries: &[EntryView], version: &str, bin_name: &str) -> Option<PathBuf> {
    let newest = |same_version: bool| {
        entries
            .iter()
            .filter(|entry| {
                entry.bin_name == bin_name && (entry.version == version) == same_version
            })
            .max_by_key(|entry| entry.modified)
            .map(|entry| entry.path.clone())
    };

    newest(true).or_else(|| newest(false))
}
