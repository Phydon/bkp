// hide console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

// TODO use different filetype instead of bkp.txt?
// TODO how to handle hidden files -> include in backup or skip?
extern crate fs_extra;
use chrono::Local;
use flexi_logger::{detailed_format, Duplicate, FileSpec, Logger};
use fs_extra::{copy_items, dir};
use log::{error, info, warn};

use std::{
    collections::BTreeMap,
    fs::{self, File},
    io::{self, BufRead, BufReader},
    path::PathBuf,
    process,
};

fn main() {
    // creates or gets the bkp folder in the config directory
    let config_dir = check_create_config_dir().unwrap_or_else(|err| {
        error!("Unable to find or create a config directory: {err}");
        process::exit(1);
    });

    // initialize the logger
    init_logger(&config_dir);

    let datetime = get_datetime();

    // read sources to back up from bkp.txt in bkp folder
    // if no bkp.txt exists in bkp folder -> create a default one
    let sources = read_sources_from_file(&config_dir).unwrap_or_else(|err| {
        error!("Unable to find or read sources: {err}");
        process::exit(1);
    });

    // create backups for the sources from bkp.txt
    // if overwrite is set to false
    // a new folder gets created with the current datetime in the name
    for (name, (src, dest, overwrite)) in sources {
        match mk_bkp(&name, &src, &dest, overwrite) {
            Ok(_) => {
                info!("{}: successfully secured, {}", name, datetime);
            }
            Err(err) => match err.kind {
                fs_extra::error::ErrorKind::NotFound => {
                    warn!(
                        "Unable to find {}, {} or {}: {}",
                        name,
                        src.display(),
                        dest.display(),
                        err
                    );
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

    // TODO remove?
    // clean up all empty directories in the bkp folder
    // if let Err(err) = clean_empty(&config_dir) {
    //     error!("Error while cleaning up bkp folder: {err}");
    //     process::exit(1);
    // }
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

fn init_logger(config_dir: &PathBuf) {
    let _logger = Logger::try_with_str("info") // log info, warn and error
        .unwrap()
        .format_for_files(detailed_format) // use timestamp for every log
        .log_to_file(
            FileSpec::default()
                .directory(&config_dir)
                .suppress_timestamp(),
        ) // change directory for logs, no timestamps in the filename
        .append() // use only one logfile
        .duplicate_to_stderr(Duplicate::Info) // print infos, warnings and errors also to the console
        .start()
        .unwrap();
}

fn get_datetime() -> String {
    Local::now().format("%a %e.%b.%Y, %T").to_string()
}

fn read_sources_from_file(
    config_dir: &PathBuf,
) -> io::Result<BTreeMap<String, (PathBuf, PathBuf, bool)>> {
    let mut bkp_path = PathBuf::new();
    bkp_path.push(&config_dir);
    bkp_path.push("bkp.txt");

    if !bkp_path.as_path().exists() {
        let default_content = format!(
            "# {}\n# {}\n# {}\n\n# {}\n# {}\n# {}",
            "Usage:",
            "<folder_name> = <path_to_source>, <path_to_destination>, <overwrite>",
            "If <path_to_destination> is \"default\", the backup will be stored in the config folder",
            "Example:",
            "my_default_backup = C:/Users/Username/path_to_source/important_folder/, default, true",
            "my_google_drive_backup = C:/Users/Username/path_to_source/important_folder/, G:/Google_Drive/My Storage/path_to_destination/, true"
        );
        fs::write(&bkp_path, default_content)?;
    }

    let file = File::open(&bkp_path)?;
    let reader = BufReader::new(file);
    let mut sources: BTreeMap<String, (PathBuf, PathBuf, bool)> = BTreeMap::new();

    let mut counter = 0;
    for line in reader.lines() {
        let line = line?;

        // ignore comments and empty lines
        if line.as_str().starts_with("#") || line.as_str().starts_with("//") || line.is_empty() {
            continue;
        } else {
            if let Some((name, content)) = line.split_once("=") {
                let srcs: Vec<&str> = content.split(",").collect();

                if srcs.len() != 3 {
                    error!("Found to many parameters in the config file");
                    return Err(io::Error::from(io::ErrorKind::InvalidData));
                }

                let src = srcs[0];
                let tmp_dest = srcs[1];
                let overwrite = srcs[2];

                let mut dest = String::new();
                match tmp_dest.to_lowercase().trim() {
                    "default" => {
                        dest.push_str(config_dir.as_path().to_string_lossy().to_string().as_str());
                    }
                    _ => {
                        dest.push_str(tmp_dest.trim());
                    }
                }

                match overwrite.to_lowercase().trim() {
                    "true" => {
                        sources.insert(
                            name.trim().to_string(),
                            (PathBuf::from(src.trim()), PathBuf::from(dest), true),
                        );
                        counter += 1;
                    }
                    "false" => {
                        sources.insert(
                            name.trim().to_string(),
                            (PathBuf::from(src.trim()), PathBuf::from(dest), false),
                        );
                        counter += 1;
                    }
                    _ => return Err(io::Error::from(io::ErrorKind::InvalidInput)),
                }
            } else {
                warn!("No \"=\" found");
                return Err(io::Error::from(io::ErrorKind::InvalidData));
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

#[allow(dead_code)]
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
