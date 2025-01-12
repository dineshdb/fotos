pub use rusqlite::Connection;
use std::{
    fs::exists,
    path::{Path, PathBuf},
    time,
};

use crate::DDriveError;

// Embed migration SQL in executable.
refinery::embed_migrations!("./migrations");

pub fn open_database(path: &PathBuf) -> crate::Result<Connection> {
    let mut con = Connection::open(path)?;
    con.pragma_update(None, "synchronous", &"normal")?;
    migrations::runner().run(&mut con)?;
    Ok(con)
}

// for testing
pub fn open_in_memory() -> crate::Result<Connection> {
    let mut con = Connection::open_in_memory()?;
    migrations::runner()
        .run(&mut con)
        .map_err(DDriveError::Migration)?;
    Ok(con)
}

pub struct Database {
    con: Connection,
}

impl Database {
    pub fn new(conn: Connection) -> Self {
        Self { con: conn }
    }

    pub fn add_files(&mut self, files: &[FileEntry]) -> crate::Result<()> {
        let mut stmt = self.con.prepare_cached(
            "INSERT INTO files(path, size, created_at, modified_at) VALUES (?, ?, ?, ?) ON CONFLICT DO NOTHING",
        )?;
        let tx = self.con.unchecked_transaction()?;
        for file in files {
            stmt.execute([
                file.path.clone(),
                file.size.to_string(),
                file.created_at.to_string(),
                file.modified_at.to_string(),
            ])
            .map_err(DDriveError::Sqlite)?;
        }
        tx.commit().map_err(DDriveError::Sqlite)?;
        crate::Result::Ok(())
    }

    pub fn get_files(&self) -> crate::Result<Vec<FileEntry>> {
        let mut stmt = self
            .con
            .prepare("SELECT path, size, created_at, modified_at, id FROM files")?;
        let rows: Vec<FileEntry> = stmt
            .query_map([], |row| {
                Ok(FileEntry {
                    path: row.get(0)?,
                    size: row.get(1)?,
                    created_at: row.get(2)?,
                    modified_at: row.get(3)?,
                    id: row.get(4)?,
                })
            })
            .map_err(DDriveError::Sqlite)?
            .filter_map(|x| x.ok())
            .collect();

        Ok(rows)
    }

    pub fn get_files_for_b3sum(&self) -> crate::Result<Vec<FileB3SumRow>> {
        let mut stmt = self
            .con
            .prepare("SELECT id, path, size FROM files WHERE b3sum IS NULL LIMIT 30")?;
        let rows: Vec<FileB3SumRow> = stmt
            .query_map([], |row| {
                Ok(FileB3SumRow {
                    file_id: row.get(0).ok(),
                    path: row.get(1)?,
                    b3sum: None,
                    size: row.get(2)?,
                })
            })
            .map_err(DDriveError::Sqlite)?
            .filter_map(|x| x.ok())
            .collect();

        Ok(rows)
    }

    pub fn set_b3sum(&mut self, sums: &[FileB3Sum]) -> crate::Result<()> {
        let mut stmt = self
            .con
            .prepare_cached("UPDATE files SET b3sum = ? WHERE id = ?")?;
        let tx = self.con.unchecked_transaction()?;
        for sum in sums {
            stmt.execute([sum.b3sum.clone(), sum.file_id.to_string()])
                .map_err(DDriveError::Sqlite)?;
        }
        tx.commit().map_err(DDriveError::Sqlite)?;
        crate::Result::Ok(())
    }

    pub fn delete_files(&mut self, file_ids: &[u64]) -> crate::Result<()> {
        let mut stmt = self.con.prepare_cached("DELETE FROM files WHERE id = ?")?;
        let tx = self.con.unchecked_transaction()?;
        for file_id in file_ids {
            stmt.execute([file_id]).map_err(DDriveError::Sqlite)?;
        }
        tx.commit().map_err(DDriveError::Sqlite)?;
        crate::Result::Ok(())
    }
    pub fn get_duplicates(&self) -> crate::Result<Vec<FileB3SumRow>> {
        let mut stmt = self.con.prepare(
            r#"SELECT b3sum, COUNT(*) FROM files GROUP BY b3sum HAVING COUNT(*) > 1 ORDER BY b3sum;"#,
        )?;
        let rows: Vec<FileB3SumCount> = stmt
            .query_map([], |row| {
                Ok(FileB3SumCount {
                    b3sum: row.get(0).ok().unwrap(),
                    count: row.get(1)?,
                })
            })
            .map_err(DDriveError::Sqlite)?
            .filter_map(|x| x.ok())
            .collect();

        let b3sums = rows
            .iter()
            .map(|x| format!("'{}'", x.b3sum))
            .collect::<Vec<String>>()
            .join(",");
        let query = format!(
            "SELECT id, path, b3sum, size FROM files WHERE b3sum IN ({}) ORDER BY size ASC;",
            b3sums,
        );
        let mut stmt = self.con.prepare(&query)?;

        let files: Vec<FileB3SumRow> = stmt
            .query_map([], |row| {
                Ok(FileB3SumRow {
                    file_id: row.get(0).ok(),
                    path: row.get(1)?,
                    b3sum: row.get(2).ok().unwrap(),
                    size: row.get(3)?,
                })
            })
            .map_err(DDriveError::Sqlite)?
            .filter_map(|x| x.ok())
            .collect();

        Ok(files)
    }

    pub fn backup(&self, conn: &mut Connection) -> crate::Result<()> {
        let backup = rusqlite::backup::Backup::new(&self.con, conn)?;
        backup.run_to_completion(5, time::Duration::from_millis(250), None)?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct FileEntry {
    pub id: Option<u64>,
    pub path: String,
    pub size: u64,
    pub created_at: u64,
    pub modified_at: u64,
}

#[derive(Debug)]
pub struct FileB3SumRow {
    pub file_id: Option<i64>,
    pub b3sum: Option<String>,
    pub path: String,
    pub size: u64,
}

#[derive(Debug)]
pub struct FileB3Sum {
    pub file_id: i64,
    pub b3sum: String,
}

#[derive(Debug)]
pub struct FileB3SumCount {
    pub count: i64,
    pub b3sum: String,
}

pub fn get_files_db(path: &Path) -> crate::Result<Database> {
    let fs_dir = path.join("metadata");
    if !exists(&fs_dir)? {
        return Err(DDriveError::RepoUninitialized);
    };
    let files_db = fs_dir.join("files.db").to_path_buf();
    Ok(Database::new(open_database(&files_db).unwrap()))
}
