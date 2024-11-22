use std::path::{Path, PathBuf};

use fotos::{
    db::{open_database, Database, FileB3Sum},
    walk,
};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

fn main() {
    let mut db = Database::new(open_database(Path::new("target/fotos.db")).unwrap());
    let pictures_dir = Path::new("/var/home/dineshdb/Pictures");
    walk(&mut db, pictures_dir).unwrap();

    let mut files = db.get_files_for_b3sum().unwrap();
    while files.len() > 0 {
        let sums: Vec<_> = files
            .par_iter()
            .filter_map(|f| {
                let path = pictures_dir.join(PathBuf::from(&f.path));
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

    let duplicates = db.get_duplicates().unwrap();
    dbg!(duplicates);
}
