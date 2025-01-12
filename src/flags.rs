pub mod flags {
    use std::path::PathBuf;

    xflags::xflags! {
        cmd fotos {
            optional -p, --path path: PathBuf

            cmd scan {
            }
            cmd duplicates {
            }
        }
    }
}
