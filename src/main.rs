use anyhow::{anyhow, Context, Result};
use clap::Parser;
use log::{error, info};
use regex::{Regex, RegexBuilder};
use simplelog::WriteLogger;
use std::ffi::OsStr;
use std::fs::File;
use std::path::{Path, PathBuf};
use tempfile::NamedTempFile;
use titlecase::titlecase;
use unrar::Archive;

#[derive(Parser, Debug)]
struct Args {
    #[clap(short, long)]
    source_directory: PathBuf,

    #[clap(short, long)]
    destination_directory: PathBuf,
}

fn main() -> Result<()> {
    let _log_file = set_up_logging()?;

    let args = Args::parse();

    let _file = match run(&args) {
        Ok(file) => Some(file),
        Err(e) => {
            error!("{e}");
            None
        }
    };

    Ok(())
}

fn set_up_logging() -> Result<File> {
    let log_file = NamedTempFile::new().context("Failed to create log file")?;
    let read_handle = log_file
        .reopen()
        .context("Failed to create read handle for log file")?;
    WriteLogger::init(
        simplelog::LevelFilter::Info,
        simplelog::Config::default(),
        log_file,
    )
    .context("Failed to initialize logger")?;

    Ok(read_handle)
}

fn run(args: &Args) -> Result<String> {
    verify_paths(args)?;
    info!("Verified paths");

    let rar_file = find_rar_file(&args.source_directory)?;
    info!("Found rar file: {:?}", rar_file);

    let destination_file_name = get_destination_file_name(&rar_file)?;
    info!(
        "Determined destination file name: {:?}",
        destination_file_name
    );

    extract_rar_file(
        &rar_file,
        &args.destination_directory,
        &destination_file_name,
    )?;
    info!("Extracted rar file");

    Ok(destination_file_name)
}

fn verify_paths(args: &Args) -> Result<()> {
    if !args.source_directory.is_dir() {
        return Err(anyhow!("Source directory is not a directory"));
    }

    if !args.destination_directory.is_dir() {
        return Err(anyhow!("Destination directory is not a directory"));
    }

    Ok(())
}

fn find_rar_file(source_directory: &Path) -> Result<PathBuf> {
    source_directory
        .read_dir()
        .context("Failed to read source directory")?
        .flatten()
        .filter_map(|entry| {
            entry
                .path()
                .extension()
                .and_then(OsStr::to_str)
                .and_then(|ext| {
                    if ext == "rar" {
                        Some(entry.path())
                    } else {
                        None
                    }
                })
        })
        .next()
        .ok_or(anyhow!("Failed to find rar file"))
}

fn get_destination_file_name(rar_file: &Path) -> Result<String> {
    let file_name = rar_file
        .file_stem()
        .and_then(OsStr::to_str)
        .ok_or(anyhow!("Failed to get rar file stem"))?;

    if let Some(episode_captures) =
        Regex::new(r"(?P<name>.*)[sS](?P<season>\d{1,2}).?[eE](?P<episode>\d{1,2})")
            .context("Failed to compile episode regex")?
            .captures(file_name)
    {
        let name = episode_captures
            .name("name")
            .map(|name| titlecase(name.as_str().replace('.', " ").trim()))
            .ok_or(anyhow!("Failed to get episode name from file name"))?;

        let season = episode_captures
            .name("season")
            .map(|season| season.as_str())
            .ok_or(anyhow!("Failed to get episode season from file name"))?;

        let episode = episode_captures
            .name("episode")
            .map(|episode| episode.as_str())
            .ok_or(anyhow!("Failed to get episode number from file name"))?;

        Ok(format!("{} - S{:02}E{:02}", name, season, episode))
    } else if let Some(movie_captures) = RegexBuilder::new(r"(?P<name>.*)\.(?P<year>\d{4})")
        .swap_greed(true)
        .build()
        .context("Failed to compile movie regex")?
        .captures(file_name)
    {
        let name = movie_captures
            .name("name")
            .map(|name| titlecase(name.as_str().replace('.', " ").trim()))
            .ok_or(anyhow!("Failed to get movie name from file name"))?;

        let year = movie_captures
            .name("year")
            .map(|year| year.as_str())
            .ok_or(anyhow!("Failed to get movie year from file name"))?;

        Ok(format!("{} ({})", name, year))
    } else {
        Err(anyhow!(
            "Failed to get destination file name from rar file stem"
        ))
    }
}

fn extract_rar_file(rar_file: &Path, destination_directory: &Path, file_name: &str) -> Result<()> {
    let mut archive = Archive::new(rar_file)
        .open_for_processing()
        .context("Failed to open rar file for processing")?;

    while let Some(header) = archive.read_header().context("Failed to read rar")? {
        archive = if header.entry().is_file() {
            let file_extension = header
                .entry()
                .filename
                .extension()
                .and_then(OsStr::to_str)
                .map(str::to_string)
                .ok_or(anyhow!("Failed to get file extension from rar header"))?;

            let destination = destination_directory
                .join(file_name)
                .with_extension(file_extension);

            if destination.exists() {
                if header.entry().unpacked_size
                    != destination
                        .metadata()
                        .map(|metadata| metadata.len() as usize)
                        .context("Failed to read file size of existing destination file")?
                {
                    std::fs::remove_file(&destination)
                        .context("Failed to remove existing destination file")?;
                    info!("Removed existing destination file: {:?}", destination)
                } else {
                    info!("Skipping existing destination file: {:?}", destination);
                    break;
                }
            }

            header
                .extract_to(destination)
                .context("Failed to extract rar file")?
        } else {
            header.skip().context("Failed to skip rar file header")?
        };
    }

    Ok(())
}
