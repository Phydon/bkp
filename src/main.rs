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

    // // FOR TESTING
    // let mut test_dir = PathBuf::new();
    // let cur_dir = env::current_dir().unwrap();
    // test_dir.push(cur_dir);
    // test_dir.push("testdir");
    // // mk_bkp(test_dir, true, config_dir).unwrap();

    // let source_path = vec![test_dir];
    // let target_path = Path::new(&config_dir).to_path_buf();
    // mk_bkp(source_path, target_path).unwrap();

    let sources = read_sources_from_file(&config_dir).unwrap_or_else(|err| {
        warn!("Unable to find or read sources: {err}");
        process::exit(1);
    });

    for (name, (src, overwrite)) in sources {
        match mk_bkp(&src, &config_dir, overwrite) {
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
    source: &PathBuf,
    target_dir: &PathBuf,
    overwrite: bool,
) -> Result<(), fs_extra::error::Error> {
    // TODO always skip existing files?
    // -> maybe use chrono for different folder/file names instead?
    let options = dir::CopyOptions {
        overwrite: overwrite,
        skip_exist: true,
        copy_inside: false,
        content_only: false,
        ..Default::default()
    };

    let mut paths = Vec::new();
    paths.push(source.as_path());
    copy_items(&paths, target_dir, &options)?;

    Ok(())
}

fn read_sources_from_file(config_dir: &PathBuf) -> io::Result<BTreeMap<String, (PathBuf, bool)>> {
    let mut bkp_path = PathBuf::new();
    bkp_path.push(&config_dir);
    bkp_path.push("bkp.txt");

    if !bkp_path.as_path().exists() {
        let default_content = "# Usage:\n# <folder_name> = <path_to_source> & <overwrite>\n# Example:\n# my_backup = C:\\Users\\Username\\Documents\\important_folder\\ & true";
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
                        // TODO panics -> handle error different?
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
