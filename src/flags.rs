use std::path::PathBuf;

xflags::xflags! {
    cmd fotos {
        optional -p, --path path: PathBuf
        default cmd help {
        }
        /// Scan the filesystem for new files
        cmd scan {
        }
        /// Identify duplicates in the database
        cmd duplicates {
        }
        /// Check the database for missing files
        cmd check {
        }
    }
}

impl Fotos {
    pub const HELP: &'static str = Self::HELP_;
}
