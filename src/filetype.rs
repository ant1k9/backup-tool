use crate::errors::BoxedErrorResult;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub enum Filetype {
    File,
    Directory,
}

pub fn get_filetype(path: &PathBuf) -> BoxedErrorResult<Filetype> {
    let metadata = fs::metadata(path)?;
    if metadata.file_type().is_dir() {
        Ok(Filetype::Directory)
    } else {
        Ok(Filetype::File)
    }
}
