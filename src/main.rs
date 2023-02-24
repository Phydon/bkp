// hide console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

// TODO use different filetype instead of bkp.txt?
extern crate fs_extra;
use chrono::Local;
use flexi_logger::{detailed_format, Duplicate, FileSpec, Logger};
use fs_extra::{copy_items, dir};
use log::{error, info, warn};

use std::{
    collections::BTreeMap,
    // env,
    fs::{self, File},
    io::{self, BufRead, BufReader},
    path::{Path, PathBuf},
    process,
};

fn main() {
    // creates or gets the bkp folder in the config directory
    let config_dir = check_create_config_dir().unwrap_or_else(|err| {
        error!("Unable to find or create a config directory: {err}");
        process::exit(1);
    });

    // initialize the logger
    let _logger = Logger::try_with_str("info") // log warn and error
        .unwrap()
        .format_for_files(detailed_format) // use timestamp for every log
        .log_to_file(
            FileSpec::default()
                .directory(&config_dir) // change directory for logs
                .suppress_timestamp(), // no timestamps in the filename
        )
        .append() // use only one logfile
        .duplicate_to_stderr(Duplicate::Info) // print infos, warnings and errors also to the console
        .start()
        .unwrap();

    let datetime = Local::now().format("%a %e.%b.%Y, %T").to_string();

    // read sources to back up from bkp.txt in bkp folder
    // if no bkp.txt exists in bkp folder -> create a default one
    let sources = read_sources_from_file(&config_dir).unwrap_or_else(|err| {
        warn!("Unable to find or read sources: {err}");
        process::exit(1);
    });

    // create backups for the sources from bkp.txt
    // if overwrite is set to false
    // a new folder gets created with the current datetime in the name
    for (name, (src, overwrite)) in sources {
        match mk_bkp(&name, &src, &config_dir, overwrite) {
            Ok(_) => {
                info!("{}: successfully secured, {}", name, datetime);
            }
            Err(err) => match err.kind {
                fs_extra::error::ErrorKind::NotFound => {
                    warn!("{name} not found: {err}");
                    continue;
                }
                fs_extra::error::ErrorKind::PermissionDenied => {
                    warn!("You don`t have access to the source {name}: {err}");
                    continue;
                }
                fs_extra::error::ErrorKind::AlreadyExists => {
                    warn!("{name} already exists: {err}\nRename the source to back up");
                    continue;
                }
                fs_extra::error::ErrorKind::InvalidFileName => {
                    warn!("{name} has an invalid name: {err}\nRename the source to back up");
                    continue;
                }
                _ => {
                    error!(
                        "Error while trying to back up {} at {}: {}",
                        name,
                        src.display(),
                        err
                    );
                    process::exit(1);
                }
            },
        }
    }

    // clean up all empty directories in the bkp folder
    if let Err(err) = clean_empty(&config_dir) {
        error!("Error while cleaning up bkp folder: {err}");
        process::exit(1);
    }
}

fn check_create_config_dir() -> io::Result<PathBuf> {
    let mut bkp_dir = PathBuf::new();
    match dirs::config_dir() {
        Some(config_dir) => {
            bkp_dir.push(config_dir);
            bkp_dir.push("bkp");
            if !bkp_dir.as_path().exists() {
                fs::create_dir(&bkp_dir)?;
            }
        }
        None => {
            error!("Unable to find config directory");
        }
    }

    Ok(bkp_dir)
}

fn mk_bkp(
    source_name: &str,
    source: &PathBuf,
    target_dir: &PathBuf,
    overwrite: bool,
) -> Result<(), fs_extra::error::Error> {
    // TODO always skip existing files?
    let options = dir::CopyOptions {
        overwrite: overwrite,
        skip_exist: true,
        copy_inside: false,
        content_only: false,
        ..Default::default()
    };

    let mut paths = Vec::new();
    paths.push(source.as_path());

    if overwrite {
        copy_items(&paths, target_dir, &options)?;
    } else {
        let datetime = Local::now().format("%e%b%Y_%H%M%S%f").to_string();
        let new_name = source_name.to_owned() + "_" + &datetime;
        let mut new_target_dir = PathBuf::new();
        new_target_dir.push(target_dir);
        new_target_dir.push(new_name);

        // TODO always creates the new_tagret_dir
        // -> if fs_extra::Error -> new created directory will be emtpy
        // => clean up empty direcories in bkp folder after every program run
        // is there a better way to handle that?
        if !new_target_dir.as_path().exists() {
            fs::create_dir(&new_target_dir)?;
        }

        copy_items(&paths, new_target_dir, &options)?;
    }

    Ok(())
}

fn read_sources_from_file(config_dir: &PathBuf) -> io::Result<BTreeMap<String, (PathBuf, bool)>> {
    let mut bkp_path = PathBuf::new();
    bkp_path.push(&config_dir);
    bkp_path.push("bkp.txt");

    if !bkp_path.as_path().exists() {
        let default_content = format!(
            "# {}\n# {}\n# {}\n# {}",
            "Usage:",
            "<folder_name> = <path_to_source> & <overwrite>",
            "Example:",
            "my_backup = C:\\Users\\Username\\Documents\\important_folder\\ & true"
        );
        fs::write(&bkp_path, default_content)?;
    }

    let file = File::open(&bkp_path)?;
    let reader = BufReader::new(file);
    let mut sources: BTreeMap<String, (PathBuf, bool)> = BTreeMap::new();

    let mut counter = 0;
    for line in reader.lines() {
        let line = line?;
        if line.as_str().starts_with("#") || line.as_str().starts_with("//") {
            continue;
        } else {
            // TODO handle if "=" is missing or wrong
            if let Some((name, source_path)) = line.split_once("=") {
                // TODO handle if "&" is missing or wrong
                if let Some((src, overwrite)) = source_path.split_once("&") {
                    match overwrite.to_lowercase().trim() {
                        "true" => {
                            sources.insert(
                                name.trim().to_string(),
                                (Path::new(src.trim()).to_path_buf(), true),
                            );
                            counter += 1;
                        }
                        "false" => {
                            sources.insert(
                                name.trim().to_string(),
                                (Path::new(src.trim()).to_path_buf(), false),
                            );
                            counter += 1;
                        }
                        // TODO panics -> handle error differently?
                        _ => return Err(io::Error::from(io::ErrorKind::InvalidInput)),
                    }
                }
            }
        }
    }

    if counter == 0 {
        warn!("No sources found. Nothing to back up");
        info!(
            "Place the files or folders to back up in {}",
            &bkp_path.display()
        );
    }

    Ok(sources)
}

fn clean_empty(bkp_path: &PathBuf) -> io::Result<()> {
    for entry in fs::read_dir(bkp_path)? {
        let entry = entry?;
        if let Ok(filetype) = entry.file_type() {
            if filetype.is_dir() {
                if fs::read_dir(entry.path())?.count() == 0 {
                    fs::remove_dir(entry.path())?; // only removes empty directories
                    info!("Removed: {}", entry.path().display());
                }
            }
        }
    }

    Ok(())
}
