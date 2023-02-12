use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

pub fn ensure_file_exists(filename: &PathBuf) -> std::io::Result<()> {
    if !Path::new(filename).exists() {
        let mut f = fs::File::create(filename)?;
        f.write_all(b"---\nbackups:")?;
    }
    Ok(())
}

pub fn ensure_dir_exists(dir_name: &PathBuf) -> std::io::Result<()> {
    fs::create_dir_all(dir_name)
}
