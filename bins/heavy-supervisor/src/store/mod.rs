use anyhow::Context;
use record::{BuildRecord, BuildState};
use select::EntryView;
use std::path::{Path, PathBuf};

pub mod record;
pub mod select;

const KNOWN_BIN_NAMES: [&str; 2] = ["panel-rs", "panel-rs-aio"];

const STATE_DIR: &str = ".state";

pub const KEEP_CACHE_ENTRIES: usize = 3;
pub const KEEP_BUILD_RECORDS: usize = 10;
pub const KEEP_FAILURE_MEMOS: usize = 10;

pub fn entry_dir(binaries: &Path, version: &str, key: &str) -> PathBuf {
    binaries
        .join(crate::cache_key::sanitize_version(version))
        .join(key)
}

fn failed_dir(binaries: &Path) -> PathBuf {
    binaries.join(STATE_DIR).join("failed")
}

fn builds_dir(binaries: &Path) -> PathBuf {
    binaries.join(STATE_DIR).join("builds")
}

pub fn failure_memo_dir(binaries: &Path, key: &str) -> PathBuf {
    let mut components = Path::new(key).components();
    let confined = match (components.next(), components.next()) {
        (Some(std::path::Component::Normal(name)), None) => name,
        _ => std::ffi::OsStr::new("unusable"),
    };

    failed_dir(binaries).join(confined)
}

pub fn build_record_dir(binaries: &Path, build_id: u64) -> PathBuf {
    builds_dir(binaries).join(build_id.to_string())
}

pub fn write_failure_memo(binaries: &Path, record: &BuildRecord) -> anyhow::Result<()> {
    clear_failure_memo(binaries, &record.cache_key)?;

    let memo = failure_memo_dir(binaries, &record.cache_key);
    record::write_record(&memo, record)?;

    let log = build_record_dir(binaries, record.build_id).join(record::BUILD_LOG_FILE);
    if log.is_file() {
        std::fs::copy(&log, memo.join(record::BUILD_LOG_FILE))
            .with_context(|| format!("copying {} into {}", log.display(), memo.display()))?;
    }

    Ok(())
}

pub fn read_failure_memo(binaries: &Path, key: &str) -> Option<BuildRecord> {
    let record = record::read_record(&failure_memo_dir(binaries, key))?;

    (record.cache_key == key && matches!(record.state, BuildState::Failed)).then_some(record)
}

pub fn clear_failure_memo(binaries: &Path, key: &str) -> anyhow::Result<()> {
    let memo = failure_memo_dir(binaries, key);

    match std::fs::remove_dir_all(&memo) {
        Err(err) if err.kind() != std::io::ErrorKind::NotFound => {
            Err(err).with_context(|| format!("clearing {}", memo.display()))
        }
        _ => Ok(()),
    }
}

fn list_builds(binaries: &Path) -> anyhow::Result<Vec<(u64, PathBuf)>> {
    let dir = builds_dir(binaries);
    if !dir.is_dir() {
        return Ok(Vec::new());
    }

    let mut builds = Vec::new();
    for entry in std::fs::read_dir(&dir)? {
        let entry = entry?;
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }

        if let Ok(id) = entry.file_name().to_string_lossy().parse::<u64>() {
            builds.push((id, path));
        }
    }

    builds.sort_unstable_by_key(|(id, _)| std::cmp::Reverse(*id));

    Ok(builds)
}

pub fn list_build_ids(binaries: &Path) -> anyhow::Result<Vec<u64>> {
    Ok(list_builds(binaries)?
        .into_iter()
        .map(|(id, _)| id)
        .collect())
}

pub fn next_build_id(binaries: &Path) -> anyhow::Result<u64> {
    Ok(list_builds(binaries)?
        .first()
        .map_or(1, |(id, _)| id.saturating_add(1)))
}

pub fn prune_state(
    binaries: &Path,
    keep_builds: usize,
    keep_failures: usize,
) -> anyhow::Result<()> {
    for (_, dir) in list_builds(binaries)?.into_iter().skip(keep_builds) {
        std::fs::remove_dir_all(&dir).with_context(|| format!("pruning {}", dir.display()))?;
    }

    let failed = failed_dir(binaries);
    if !failed.is_dir() {
        return Ok(());
    }

    let mut memos: Vec<(PathBuf, std::time::SystemTime)> = Vec::new();
    for entry in std::fs::read_dir(&failed)? {
        let entry = entry?;
        if !entry.path().is_dir() {
            continue;
        }

        memos.push((entry.path(), entry_mtime(&entry.path())?));
    }

    memos.sort_by_key(|(_, modified)| std::cmp::Reverse(*modified));

    for (path, _) in memos.into_iter().skip(keep_failures) {
        std::fs::remove_dir_all(&path).with_context(|| format!("pruning {}", path.display()))?;
    }

    Ok(())
}

pub fn list_entries(binaries: &Path) -> anyhow::Result<Vec<EntryView>> {
    let mut entries = Vec::new();

    for version in std::fs::read_dir(binaries)? {
        let version = version?;
        if !version.path().is_dir() || version.file_name() == STATE_DIR {
            continue;
        }
        let dir_version = version.file_name().to_string_lossy().into_owned();

        for key in std::fs::read_dir(version.path())? {
            let key = key?;
            let dir = key.path();
            if !dir.is_dir() {
                continue;
            }
            let dir_key = key.file_name().to_string_lossy().into_owned();

            let entry = match record::read_record(&dir) {
                Some(record) => {
                    if !dir.join(&record.bin_name).is_file() {
                        continue;
                    }
                    let verified = record.verified
                        && record.cache_key == dir_key
                        && crate::cache_key::sanitize_version(&record.panel_version) == dir_version;
                    EntryView {
                        path: dir.clone(),
                        version: record.panel_version,
                        key: record.cache_key,
                        bin_name: record.bin_name,
                        verified,
                        modified: entry_mtime(&dir)?,
                    }
                }
                None => {
                    let Some(bin_name) = legacy_bin_name(&dir) else {
                        continue;
                    };
                    EntryView {
                        path: dir.clone(),
                        version: dir_version.clone(),
                        key: dir_key,
                        bin_name,
                        verified: false,
                        modified: entry_mtime(&dir)?,
                    }
                }
            };

            entries.push(entry);
        }
    }

    Ok(entries)
}

pub fn install_binary(entry: &Path, bin_name: &str, source: &Path) -> anyhow::Result<PathBuf> {
    std::fs::create_dir_all(entry).with_context(|| format!("creating {}", entry.display()))?;

    let installed = entry.join(bin_name);
    let staged = entry.join(format!("{bin_name}.installing"));

    std::fs::copy(source, &staged)
        .and_then(|_| std::fs::rename(&staged, &installed))
        .inspect_err(|_| {
            let _ = std::fs::remove_file(&staged);
        })
        .with_context(|| format!("installing {} as {}", source.display(), installed.display()))?;

    Ok(installed)
}

pub fn mark_unverified(entry: &Path) -> anyhow::Result<bool> {
    let Some(mut record) = record::read_record(entry) else {
        return Ok(false);
    };
    if !record.verified {
        return Ok(false);
    }

    record.verified = false;
    record::write_record(entry, &record)?;

    Ok(true)
}

fn legacy_bin_name(dir: &Path) -> Option<String> {
    KNOWN_BIN_NAMES
        .into_iter()
        .find(|name| dir.join(name).is_file())
        .map(str::to_string)
}

pub fn prune_entries(binaries: &Path, keep: usize) -> anyhow::Result<Vec<PathBuf>> {
    let mut entries: Vec<(PathBuf, std::time::SystemTime)> = Vec::new();

    for version in std::fs::read_dir(binaries)? {
        let version = version?;
        if !version.path().is_dir() || version.file_name() == STATE_DIR {
            continue;
        }

        for entry in std::fs::read_dir(version.path())? {
            let entry = entry?;
            if !entry.path().is_dir() {
                continue;
            }

            entries.push((entry.path(), entry_mtime(&entry.path())?));
        }
    }

    entries.sort_by_key(|(_, modified)| std::cmp::Reverse(*modified));

    let mut removed = Vec::new();
    for (path, _) in entries.into_iter().skip(keep) {
        std::fs::remove_dir_all(&path).with_context(|| format!("pruning {}", path.display()))?;
        removed.push(path);
    }

    Ok(removed)
}

fn entry_mtime(dir: &Path) -> anyhow::Result<std::time::SystemTime> {
    let mut newest = std::time::SystemTime::UNIX_EPOCH;

    for file in std::fs::read_dir(dir)? {
        let file = file?;
        if !file.path().is_file() {
            continue;
        }

        let modified = file.metadata()?.modified()?;
        if modified > newest {
            newest = modified;
        }
    }

    Ok(newest)
}
