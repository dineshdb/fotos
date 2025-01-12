mod flags;

use flags::{Fotos, FotosCmd};
use fotos::{
    db::{get_files_db, FileB3Sum},
    walk,
};
use itertools::Itertools;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use std::path::{Path, PathBuf};

fn main() {
    let flags = Fotos::from_env().expect("couldn't parse flags");
    let dir = flags
        .path
        .unwrap_or_else(|| PathBuf::from("."))
        .canonicalize()
        .unwrap();

    match flags.subcommand {
        FotosCmd::Scan(_) => {
            println!("Scanning {}", dir.display());
            scan_dir(&dir);
        }
        FotosCmd::Duplicates(_) => {
            println!("Finding duplicates in {}", dir.display());
            find_duplicates(&dir);
        }
        FotosCmd::Check(_) => {
            println!("Checking {}", dir.display());
            let mut db = get_files_db(&dir);
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
        FotosCmd::Help(_) => {
            println!("{}", Fotos::HELP);
        }
    }
}

fn scan_dir(path: &Path) {
    let mut db = get_files_db(&path);
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
    let db = get_files_db(&path);
    let duplicates = db.get_duplicates().unwrap();
    let grouped = duplicates.iter().into_group_map_by(|r| r.b3sum.clone());
    for (b3sum, files) in &grouped {
        let mut b3sum = b3sum.clone().unwrap_or_default();
        let _ = b3sum.split_off(10);
        println!("{}:", b3sum);
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
