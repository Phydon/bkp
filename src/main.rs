// hide console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

extern crate fs_extra;
// use chrono::Local;
use flexi_logger::{detailed_format, Duplicate, FileSpec, Logger};
use fs_extra::{copy_items, dir};
use log::{error, warn};

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
    let _logger = Logger::try_with_str("warn") // log warn and error
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

    // // FIXME still testing
    // let mut test_dir = PathBuf::new();
    // let cur_dir = env::current_dir().unwrap();
    // test_dir.push(cur_dir);
    // test_dir.push("testdir");
    // // mk_bkp(test_dir, true, config_dir).unwrap();

    // let source_path = vec![test_dir];
    // let target_path = Path::new(&config_dir).to_path_buf();
    // mk_bkp(source_path, target_path).unwrap();

    let sources = read_sources_from_file(&config_dir).unwrap_or_else(|err| {
        warn!("Unable to find sources: {err}");
        process::exit(1);
    });

    for (name, src) in sources {
        println!("{}: {}", name, src.display());
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

    // let dir = bkp_dir.into_os_string().into_string().unwrap();
    // Ok(dir)
    Ok(bkp_dir)
}

// TODO make sure not to override already existing backup folders/files
// -> use chrono for different folder/file names?
fn mk_bkp(paths: Vec<PathBuf>, target_dir: PathBuf) -> Result<(), fs_extra::error::Error> {
    // let options = dir::CopyOptions::new();
    let options = dir::CopyOptions {
        overwrite: false,
        skip_exist: false,
        copy_inside: false,
        content_only: false,
        ..Default::default()
    };
    copy_items(&paths, target_dir, &options)?;

    Ok(())
}

fn read_sources_from_file(config_dir: &PathBuf) -> io::Result<BTreeMap<String, PathBuf>> {
    let mut bkp_path = PathBuf::new();
    bkp_path.push(&config_dir);
    bkp_path.push("bkp.txt");

    if !bkp_path.as_path().exists() {
        let default_content = "# Usage:\n# <folder_name> = <path_to_source>\n# Example:\n# my_backup = ~/Documents/important_folder/";
        fs::write(&bkp_path, default_content)?;
    }

    let file = File::open(bkp_path)?;
    let reader = BufReader::new(file);
    let mut sources = BTreeMap::new();

    for line in reader.lines() {
        let line = line?;
        if !line.contains("#") {
            if let Some((name, src)) = line.split_once("=") {
                sources.insert(name.trim().to_string(), Path::new(src.trim()).to_path_buf());
            } else {
                // TODO
                // create a helpful msg where to find the bkp.txt
                // if no sources are in this file
                warn!("No sources found");
            }
        }
    }

    Ok(sources)
}
