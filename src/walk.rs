use crate::{
    db::{self, Database},
    DDriveError,
};
use anyhow::Ok;
use std::{fs, path::Path, time::UNIX_EPOCH};

pub fn walk(db: &mut Database, base_dir: &Path, path: &Path) -> anyhow::Result<()> {
    let entries: Vec<_> = fs::read_dir(path)?
        .into_iter()
        .filter_map(|f| f.ok())
        .collect();
    let directories = entries
        .iter()
        .filter(|e| e.file_type().map(|t| t.is_dir()).unwrap_or(false))
        .collect::<Vec<_>>();
    let files = entries
        .iter()
        .filter(|e| e.file_type().map(|t| t.is_file()).unwrap_or(false))
        .filter_map(|entry| {
            let meta = entry.metadata().ok()?;
            if entry.path().starts_with(".") {
                return None;
            }
            Some(db::FileEntry {
                id: None,
                path: entry
                    .path()
                    .strip_prefix(base_dir)
                    .ok()?
                    .to_string_lossy()
                    .to_string(),
                size: meta.len(),
                created_at: meta
                    .created()
                    .ok()?
                    .duration_since(UNIX_EPOCH)
                    .map_err(DDriveError::SystemTime)
                    .ok()?
                    .as_secs(),
                modified_at: meta
                    .modified()
                    .ok()?
                    .duration_since(UNIX_EPOCH)
                    .map_err(DDriveError::SystemTime)
                    .ok()?
                    .as_secs(),
            })
        })
        .collect::<Vec<_>>();
    for dir in directories {
        walk(db, base_dir, &dir.path())?;
    }
    db.add_files(files.as_slice())?;

    Ok(())
}
