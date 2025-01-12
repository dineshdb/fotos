use std::path::PathBuf;

xflags::xflags! {
    cmd fotos {
        optional -r, --repo repo: PathBuf
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

        // All of these commands work on the single snapshot of files
        // we don't have any notion of versioning or backup, unlike git or restic

        /// Initialize the repo
        cmd init {}

        /// Add files to the repo. Files are added incrementally
        cmd add {
            repeated files:PathBuf
        }

        /// Remove files from the repo. Files are removed incrementally
        cmd rm {
            repeated -f, --files files:PathBuf
        }
    }
}

impl Fotos {
    pub const HELP: &'static str = Self::HELP_;
}
