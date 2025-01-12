mod flags;

use flags::{Fotos, FotosCmd};
use fotos::{
    db::{get_files_db, FileB3Sum},
    walk, Result,
};
use itertools::Itertools;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use std::path::{Path, PathBuf};

fn main() -> Result<()> {
    let flags = Fotos::from_env().expect("couldn't parse flags");
    let repo_dir = flags
        .repo
        .unwrap_or_else(|| PathBuf::from("."))
        .canonicalize()
        .unwrap();

    match flags.subcommand {
        FotosCmd::Scan(_) => {
            println!("Scanning {}", repo_dir.display());
            scan_dir(&repo_dir)?;
        }
        FotosCmd::Duplicates(_) => {
            println!("Finding duplicates in {}", repo_dir.display());
            find_duplicates(&repo_dir)?;
        }
        FotosCmd::Check(_) => {
            println!("Checking {}", repo_dir.display());
            let mut db = get_files_db(&repo_dir)?;
            let files = db.get_files().unwrap();
            let mut deleted_files = Vec::new();
            for file in files {
                let path = repo_dir.join(PathBuf::from(&file.path));
                if !path.exists() {
                    deleted_files.push(file.id.unwrap());
                }
            }
            if deleted_files.len() > 0 {
                db.delete_files(&deleted_files).unwrap();
            }
        }
        FotosCmd::Help(_) => {
            println!("{}", Fotos::HELP);
        }
        FotosCmd::Init(_) => {
            println!("Initializing {}", repo_dir.display());
            std::fs::create_dir_all(repo_dir.join("objects"))?;
            std::fs::create_dir_all(repo_dir.join("metadata"))?;
        }
        FotosCmd::Add(add) => {
            println!("Adding files to {}", repo_dir.display());
            let _db = get_files_db(&repo_dir)?;

            for file in add.files {
                println!("{}", file.display());
                // todo: add file to db
                // this is kind of complicated than just calling walk(db, base_path, folder) due to
                // - parameter can be file and folder both
                // - we support calling add from anywhere so the base_path can't be repo_dir
                // - we also need to copy the file to the repo objects
            }
        }
        FotosCmd::Rm(_) => {
            println!("Removing files from {}", repo_dir.display());
            todo!("remove files from the repo");
        }
    };
    Ok(())
}

fn scan_dir(path: &Path) -> Result<()> {
    let mut db = get_files_db(&path)?;
    walk(&mut db, path, path).unwrap();
    let mut files = db.get_files_for_b3sum().unwrap();
    while files.len() > 0 {
        let sums: Vec<_> = files
            .par_iter()
            .filter_map(|f| {
                let path = path.join(PathBuf::from(&f.path));
                let sum = fotos::b3sum::b3sum(&path).unwrap();
                Some(FileB3Sum {
                    file_id: f.file_id.unwrap(),
                    b3sum: sum,
                })
            })
            .collect();

        db.set_b3sum(&sums).unwrap();
        files = db.get_files_for_b3sum().unwrap();
    }
    Ok(())
}

fn find_duplicates(path: &Path) -> crate::Result<()> {
    let db = get_files_db(&path)?;
    let duplicates = db.get_duplicates().unwrap();
    let grouped = duplicates.iter().into_group_map_by(|r| r.b3sum.clone());
    for entry in &duplicates {
        let b3sum = &entry.b3sum.clone().unwrap_or_default();
        println!("{:.6} {} {}", b3sum, entry.size, entry.path);
    }

    let size_saved: u64 = grouped
        .iter()
        .map(|(_, files)| files.first().unwrap().size * (files.len() as u64 - 1))
        .sum();
    println!("Total size saved: {}", size_saved);
    Ok(())
}
