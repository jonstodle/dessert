use anyhow::{anyhow, Context, Result};
use clap::Parser;
use regex::{Regex, RegexBuilder};
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use unrar::Archive;

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
    verify_paths(args)?;

    let rar_file = find_rar_file(&args.source_directory)?;

    let destination_file_name = get_destination_file_name(&rar_file)?;

    extract_rar_file(
        &rar_file,
        &args.destination_directory,
        &destination_file_name,
    )?;

    Ok(())
}

fn verify_paths(args: &Args) -> Result<()> {
    if !args.source_directory.is_dir() {
        return Err(anyhow!("source directory is not a directory"));
    }

    if !args.destination_directory.is_dir() {
        return Err(anyhow!("destination directory is not a directory"));
    }

    Ok(())
}

fn find_rar_file(source_directory: &Path) -> Result<PathBuf> {
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

fn get_destination_file_name(rar_file: &Path) -> Result<String> {
    let file_name = rar_file
        .file_stem()
        .and_then(OsStr::to_str)
        .ok_or(anyhow!("failed to get rar file stem"))?;

    if let Some(episode_captures) =
        Regex::new(r"(?P<name>.*)[sS](?P<season>\d{1,2}).?[eE](?P<episode>\d{1,2})")
            .context("failed to compile episode regex")?
            .captures(file_name)
    {
        let name = episode_captures
            .name("name")
            .map(|name| name.as_str().replace('.', " ").trim().to_string())
            .ok_or(anyhow!("failed to get episode name from file name"))?;

        let season = episode_captures
            .name("season")
            .map(|season| season.as_str())
            .ok_or(anyhow!("failed to get episode season from file name"))?;

        let episode = episode_captures
            .name("episode")
            .map(|episode| episode.as_str())
            .ok_or(anyhow!("failed to get episode number from file name"))?;

        Ok(format!("{} - S{:02}E{:02}", name, season, episode))
    } else if let Some(movie_captures) = RegexBuilder::new(r"(?P<name>.*)\.(?P<year>\d{4})")
        .swap_greed(true)
        .build()
        .context("failed to compile movie regex")?
        .captures(file_name)
    {
        let name = movie_captures
            .name("name")
            .map(|name| name.as_str().replace('.', " ").trim().to_string())
            .ok_or(anyhow!("failed to get movie name from file name"))?;

        let year = movie_captures
            .name("year")
            .map(|year| year.as_str())
            .ok_or(anyhow!("failed to get movie year from file name"))?;

        Ok(format!("{} ({})", name, year))
    } else {
        Err(anyhow!(
            "failed to get destination file name from rar file stem"
        ))
    }
}

fn extract_rar_file(rar_file: &Path, destination_directory: &Path, file_name: &str) -> Result<()> {
    let mut archive = Archive::new(rar_file)
        .open_for_processing()
        .context("failed to open rar file for processing")?;

    while let Some(header) = archive.read_header().context("failed to read rar")? {
        archive = if header.entry().is_file() {
            let file_extension = header
                .entry()
                .filename
                .extension()
                .and_then(OsStr::to_str)
                .map(str::to_string)
                .ok_or(anyhow!("failed to get file extension from rar header"))?;

            header
                .extract_to(
                    destination_directory
                        .join(file_name)
                        .with_extension(file_extension),
                )
                .context("failed to extract rar file")?;

            break;
        } else {
            header.skip().context("failed to skip rar file header")?
        };
    }

    Ok(())
}
