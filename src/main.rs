use dirs;
use std::env;
use std::fs::create_dir_all;
use std::path::{Path, PathBuf};
use structopt::StructOpt;

const BACKUP_TOOL_DIRECTORY: &str = ".local/share/backup-tool";

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

fn do_backup(backup_directory: PathBuf, from: PathBuf) {}
fn list(backup_directory: PathBuf) {}
fn versions(backup_directory: PathBuf, from: PathBuf) {}
fn restore(backup_directory: PathBuf, to: PathBuf, version: Option<String>) {}
fn clean(backup_directory: PathBuf, to: PathBuf, version: Option<String>) {}

fn ensure_exists(backup_directory: PathBuf) -> std::io::Result<()> {
    create_dir_all(backup_directory)
}

fn main() -> std::io::Result<()> {
    let default_path = dirs::home_dir()
        .unwrap()
        .join(Path::new(BACKUP_TOOL_DIRECTORY));
    let backup_directory =
        env::var("BACKUP_TOOL_DIRECTORY").map_or(default_path, |v| PathBuf::from(v));
    ensure_exists(backup_directory)?;

    let opt = Opt::from_args();
    match opt {
        Opt::The(the) => println!("the"),
        Opt::List(_) => println!("the"),
        Opt::Versions(versions) => println!("the"),
        Opt::Restore(restore) => println!("the"),
        Opt::Clean(clean) => println!("the"),
    }

    Ok(())
}
