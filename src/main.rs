use anyhow::{anyhow, Context, Result};
use clap::Parser;
use std::ffi::OsStr;
use std::path::PathBuf;

#[derive(Parser, Debug)]
struct Args {
    #[clap(short, long)]
    source_directory: PathBuf,

    #[clap(short, long)]
    destination_directory: PathBuf,
}

fn main() -> Result<()> {
    let args = Args::parse();

    if let Err(_e) = run(&args) {
        todo!("send error mail (maybe include logs)")
    }

    Ok(())
}

fn run(args: &Args) -> Result<()> {
    let rar_file = find_rar_file(&args.source_directory)?;

    Ok(())
}

fn find_rar_file(source_directory: &PathBuf) -> Result<PathBuf> {
    source_directory
        .read_dir()
        .context("failed to read source directory")?
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| path.is_file())
        .filter_map(|path| {
            if path.extension().and_then(OsStr::to_str).is_some() {
                Some(path)
            } else {
                None
            }
        })
        .next()
        .ok_or(anyhow!("failed to find rar file"))
}
