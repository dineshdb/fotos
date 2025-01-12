use std::path::{Path, PathBuf};
mod flags;

use fotos::{
    db::{open_database, Database, FileB3Sum},
    walk,
};
use itertools::Itertools;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

fn main() {
    let flags = flags::flags::Fotos::from_env().expect("couldn't parse flags");
    let dir = flags
        .path
        .unwrap_or_else(|| PathBuf::from("."))
        .canonicalize()
        .unwrap();

    match flags.subcommand {
        flags::flags::FotosCmd::Scan(_) => {
            std::fs::create_dir_all(dir.join(".fs")).unwrap();
            println!("Scanning {}", dir.display());
            scan_dir(&dir);
        }
        flags::flags::FotosCmd::Duplicates(_) => {
            std::fs::create_dir_all(dir.join(".fs")).unwrap();
            println!("Finding duplicates in {}", dir.display());
            find_duplicates(&dir);
        }
        flags::flags::FotosCmd::Check(_) => {
            println!("Checking {}", dir.display());
            let mut db =
                Database::new(open_database(dir.join(".fs").join("files.db").as_path()).unwrap());
            let files = db.get_files().unwrap();
            let mut deleted_files = Vec::new();
            for file in files {
                let path = dir.join(PathBuf::from(&file.path));
                if !path.exists() {
                    deleted_files.push(file.id.unwrap());
                }
            }
            if deleted_files.len() > 0 {
                db.delete_files(&deleted_files).unwrap();
            }
        }
    }
}

fn scan_dir(path: &Path) {
    let mut db = Database::new(open_database(path.join(".fs").join("files.db").as_path()).unwrap());

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
}

fn find_duplicates(path: &Path) {
    let db = Database::new(open_database(path.join(".fs").join("files.db").as_path()).unwrap());
    let duplicates = db.get_duplicates().unwrap();
    let grouped = duplicates.iter().into_group_map_by(|r| r.b3sum.clone());
    for (b3sum, files) in &grouped {
        println!("{}:", b3sum.clone().unwrap_or_default().split_off(10),);
        for duplicate in files {
            println!("  {} {}", duplicate.size, duplicate.path);
        }
    }

    let size_saved: u64 = grouped
        .iter()
        .map(|(_, files)| files.first().unwrap().size * (files.len() as u64 - 1))
        .sum();
    println!("Total size saved: {}", size_saved);
}
