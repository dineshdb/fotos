use blake3::Hasher;
use std::fs::File;
use std::io::{self, Read};
use std::path::PathBuf;

pub fn b3sum(path: &PathBuf) -> io::Result<String> {
    let mut file = File::open(path)?;
    let mut hasher = Hasher::new();

    let mut buffer = [0; 4096]; // Buffer size of 4KB
    loop {
        let bytes_read = file.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }

    // Finalize the hash and print it
    let hash = hasher.finalize();

    dbg!(format!("{} {}", path.display(), hash.to_hex()));

    Ok(format!("{}", hash.to_hex()))
}
