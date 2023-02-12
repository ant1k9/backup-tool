use crate::errors::BoxedErrorResult;
use crate::file_ops::ensure_file_exists;
use crate::filetype::Filetype;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

const BACKUP_METADATA: &str = "backup.yaml";

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BackupVersionMetadata {
    pub version: u32,
    pub timestamp: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BackupMetadata {
    pub path: PathBuf,
    pub filetype: Filetype,
    pub versions: Vec<BackupVersionMetadata>,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Metadata {
    pub backups: HashMap<PathBuf, BackupMetadata>,
}

pub fn read_metadata(backup_directory: &Path) -> BoxedErrorResult<Metadata> {
    let backup_metadata_file = backup_directory.join(BACKUP_METADATA);
    ensure_file_exists(&backup_metadata_file)?;

    let f = fs::File::open(backup_metadata_file)?;
    let backup_metadata: Metadata =
        serde_yaml::from_reader(f).expect("cannot read backup metadata");
    Ok(backup_metadata)
}

pub fn save_metadata(backup_directory: &Path, backup_metadata: &Metadata) -> BoxedErrorResult<()> {
    let backup_metadata_file = backup_directory.join(BACKUP_METADATA);
    let f =
        fs::File::create(backup_metadata_file).expect("cannot open file to save updated metadata");
    serde_yaml::to_writer(f, backup_metadata).expect("failed to write updated metadata");
    Ok(())
}
