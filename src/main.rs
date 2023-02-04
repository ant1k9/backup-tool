use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::{env, fs};
use structopt::StructOpt;

const BACKUP_TOOL_DIRECTORY: &str = ".local/share/backup-tool";
const BACKUP_METADATA: &str = "backup.yaml";

pub type BoxedErrorResult<T> = Result<T, Box<dyn std::error::Error>>;

#[derive(Debug, StructOpt)]
pub struct The {
    path: PathBuf,
}

#[derive(Debug, StructOpt)]
pub struct List {}

#[derive(Debug, StructOpt)]
pub struct Versions {
    path: PathBuf,
}

#[derive(Debug, StructOpt)]
pub struct Restore {
    #[structopt(short, long)]
    version: Option<String>,
    path: PathBuf,
}

#[derive(Debug, StructOpt)]
pub struct Clean {
    #[structopt(short, long)]
    version: Option<String>,
    path: PathBuf,
}

#[derive(StructOpt, Debug)]
#[structopt(name = "backup-tool")]
enum Opt {
    #[structopt(name = "the")]
    The(The),
    #[structopt(name = "list")]
    List(List),
    #[structopt(name = "versions")]
    Versions(Versions),
    #[structopt(name = "restore")]
    Restore(Restore),
    #[structopt(name = "clean")]
    Clean(Clean),
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub enum Filetype {
    File,
    Directory,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BackupVersionMetadata {
    version: u32,
    timestamp: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BackupMetadata {
    path: PathBuf,
    filetype: Filetype,
    versions: Vec<BackupVersionMetadata>,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Metadata {
    backups: HashMap<PathBuf, BackupMetadata>,
}

fn rand_string() -> String {
    thread_rng()
        .sample_iter(&Alphanumeric)
        .take(16)
        .map(char::from)
        .collect()
}

fn ensure_file_exists(filename: &PathBuf) -> std::io::Result<()> {
    if !Path::new(filename).exists() {
        let mut f = fs::File::create(filename)?;
        f.write_all(b"---\nbackups:")?;
    }
    Ok(())
}

fn ensure_dir_exists(dir_name: &PathBuf) -> std::io::Result<()> {
    fs::create_dir_all(dir_name)
}

fn get_filetype(path: &PathBuf) -> BoxedErrorResult<Filetype> {
    let metadata = fs::metadata(path)?;
    if metadata.file_type().is_dir() {
        Ok(Filetype::Directory)
    } else {
        Ok(Filetype::File)
    }
}

fn read_metadata(backup_directory: &Path) -> BoxedErrorResult<Metadata> {
    let backup_metadata_file = backup_directory.join(BACKUP_METADATA);
    ensure_file_exists(&backup_metadata_file)?;

    let f = fs::File::open(backup_metadata_file)?;
    let backup_metadata: Metadata =
        serde_yaml::from_reader(f).expect("cannot read backup metadata");
    Ok(backup_metadata)
}

fn save_metadata(backup_directory: &Path, backup_metadata: &Metadata) -> BoxedErrorResult<()> {
    let backup_metadata_file = backup_directory.join(BACKUP_METADATA);
    let f =
        fs::File::create(backup_metadata_file).expect("cannot open file to save updated metadata");
    serde_yaml::to_writer(f, backup_metadata).expect("failed to write updated metadata");
    Ok(())
}

fn do_backup(backup_directory: &Path, from: &PathBuf) -> BoxedErrorResult<()> {
    let mut backup_metadata: Metadata = read_metadata(backup_directory)?;
    let canonical = fs::canonicalize(from).expect("cannot canonicalize path to file");
    let from = &canonical;

    let from_filetype = get_filetype(from).expect("cannot check filetype for backuping file");

    if !backup_metadata.backups.contains_key(from) {
        let backup_metadata_path = backup_directory.join(rand_string());
        ensure_dir_exists(&backup_metadata_path).expect("failed to create folder for backups");
        backup_metadata.backups.insert(
            from.clone(),
            BackupMetadata {
                path: backup_metadata_path,
                filetype: from_filetype,
                versions: Vec::new(),
            },
        );
    }

    let this_backup_metadata = backup_metadata.backups.get_mut(from).unwrap();
    let zero_version = &BackupVersionMetadata {
        version: 0,
        timestamp: "".to_owned(),
    };
    let last_version = this_backup_metadata.versions.last().unwrap_or(zero_version);
    let new_version = BackupVersionMetadata {
        version: last_version.version + 1,
        timestamp: chrono::offset::Local::now().to_rfc2822(),
    };

    let new_version_path = this_backup_metadata
        .path
        .join(format!("v{:?}", new_version.version));

    match from_filetype {
        Filetype::Directory => {
            copy_dir::copy_dir(from, new_version_path).expect("failed to backup directory");
        }
        Filetype::File => {
            fs::copy(from, new_version_path).expect("failed to backup file");
        }
    }

    this_backup_metadata.versions.push(new_version);
    save_metadata(backup_directory, &backup_metadata)?;
    println!("successfully create backups for {}", from.to_str().unwrap());

    Ok(())
}

fn list(backup_directory: &Path) -> BoxedErrorResult<()> {
    let backup_metadata = read_metadata(backup_directory)?;
    backup_metadata
        .backups
        .iter()
        .for_each(|backup| println!("{}", backup.0.to_str().unwrap()));
    Ok(())
}

fn list_versions(backup_directory: &Path, from: &PathBuf) -> BoxedErrorResult<()> {
    let backup_metadata = read_metadata(backup_directory)?;
    if !backup_metadata.backups.contains_key(from) {
        println!("no backups for {}", from.to_str().unwrap());
        return Ok(());
    }

    backup_metadata
        .backups
        .get(from)
        .unwrap()
        .versions
        .iter()
        .for_each(|version| println!("v{:?} ({})", version.version, version.timestamp));
    Ok(())
}

fn restore_version(
    this_backup_metadata: &BackupMetadata,
    to: &PathBuf,
    version: u32,
) -> BoxedErrorResult<()> {
    let backup_version_path = this_backup_metadata.path.join(format!("v{:?}", version));

    match this_backup_metadata.filetype {
        Filetype::File => {
            fs::copy(backup_version_path, to).expect("cannot restore backup file");
        }
        Filetype::Directory => {
            if Path::new(to).exists() {
                fs::remove_dir_all(to).expect("failed to clean target directory");
            }
            copy_dir::copy_dir(backup_version_path, to).expect("cannot restore backup directory");
        }
    };

    println!("successfully restore backup for {}", to.to_str().unwrap());
    Ok(())
}

fn restore(backup_directory: &Path, to: &PathBuf, version: Option<String>) -> BoxedErrorResult<()> {
    let backup_metadata = read_metadata(backup_directory)?;
    let canonical = fs::canonicalize(to).expect("cannot canonicalize path to file");
    let to = &canonical;
    if !backup_metadata.backups.contains_key(to) {
        println!("no backups for {}", to.to_str().unwrap());
        return Ok(());
    }

    let this_backup_metadata = backup_metadata.backups.get(to).unwrap();
    if let Some(v) = version {
        if v.is_empty() || !v.starts_with('v') {
            return Err(
                String::from("invalid format for version, should be in form v1, v2...").into(),
            );
        }
        let version = v[1..].parse::<u32>().expect("cannot parse version number");
        match this_backup_metadata
            .versions
            .iter()
            .find(|item| item.version == version)
        {
            Some(_) => restore_version(this_backup_metadata, to, version)?,
            None => println!("no version {} found", v),
        }
        return Ok(());
    }

    if this_backup_metadata.versions.is_empty() {
        println!("no backups for {}", to.to_str().unwrap());
        return Ok(());
    }

    restore_version(
        this_backup_metadata,
        to,
        this_backup_metadata.versions[0].version,
    )
}

fn clean(backup_directory: &Path, to: &PathBuf, version: Option<String>) -> BoxedErrorResult<()> {
    let mut backup_metadata = read_metadata(backup_directory)?;
    let canonical = fs::canonicalize(to).expect("cannot canonicalize path to file");
    let to = &canonical;

    if !backup_metadata.backups.contains_key(to) {
        println!("no backups for {}", to.to_str().unwrap());
        return Ok(());
    }

    let this_backup_metadata = backup_metadata.backups.get_mut(to).unwrap();
    if let Some(v) = version {
        if v.is_empty() || !v.starts_with('v') {
            return Err(
                String::from("invalid format for version, should be in form v1, v2...").into(),
            );
        }
        let version = v[1..].parse::<u32>().expect("cannot parse version number");
        match this_backup_metadata
            .versions
            .iter()
            .find(|item| item.version == version)
        {
            Some(_) => {
                let remove_version_path = this_backup_metadata.path.join(&v);
                match this_backup_metadata.filetype {
                    Filetype::File => fs::remove_file(remove_version_path).unwrap_or_else(|_| {
                        panic!("cannot remove {v} for file {}", to.to_str().unwrap())
                    }),
                    Filetype::Directory => {
                        fs::remove_file(remove_version_path).unwrap_or_else(|_| {
                            panic!("cannot remove {v} for directory {}", to.to_str().unwrap())
                        })
                    }
                }

                this_backup_metadata
                    .versions
                    .retain(|item| item.version != version);
                save_metadata(backup_directory, &backup_metadata)?;
                println!(
                    "successfully remove version {v} for {}",
                    to.to_str().unwrap()
                );
            }
            None => println!("no version {} found", v),
        }
        return Ok(());
    }

    fs::remove_dir_all(&this_backup_metadata.path)
        .unwrap_or_else(|_| panic!("cannot clean backups for {}", to.to_str().unwrap()));

    backup_metadata.backups.remove(to);
    save_metadata(backup_directory, &backup_metadata)?;

    println!("successfully remove backups for {}", to.to_str().unwrap());
    Ok(())
}

fn main() -> BoxedErrorResult<()> {
    let default_path = dirs::home_dir()
        .unwrap()
        .join(Path::new(BACKUP_TOOL_DIRECTORY));
    let backup_directory = env::var("BACKUP_TOOL_DIRECTORY").map_or(default_path, PathBuf::from);
    ensure_dir_exists(&backup_directory)?;

    let opt = Opt::from_args();
    match opt {
        Opt::The(the) => do_backup(&backup_directory, &the.path)?,
        Opt::List(_) => list(&backup_directory)?,
        Opt::Versions(versions) => list_versions(&backup_directory, &versions.path)?,
        Opt::Restore(r) => restore(&backup_directory, &r.path, r.version)?,
        Opt::Clean(c) => clean(&backup_directory, &c.path, c.version)?,
    }

    Ok(())
}
