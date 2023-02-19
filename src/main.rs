// hide console window on Windows in release
// #![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use chrono::Local;
use flexi_logger::{detailed_format, Duplicate, FileSpec, Logger};
use log::error;

use std::{env, fs, io, path::PathBuf, process};

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
                .directory(&config_dir)
                .suppress_timestamp(),
        ) // change directory for logs, no timestamps in the filename
        .append() // use only one logfile
        .duplicate_to_stderr(Duplicate::Info) // print infos, warnings and errors also to the console
        .start()
        .unwrap();

    // FIXME still testing
    let mut test_dir = PathBuf::new();
    let cur_dir = env::current_dir().unwrap();
    test_dir.push(cur_dir);
    test_dir.push("testdir");
    mk_bkp(test_dir, true, config_dir).unwrap();
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

// TODO needed?
// fn show_log_file(config_dir: &str) -> io::Result<String> {
//     let log_path = Path::new(&config_dir).join("bkp.log");
//     match log_path.try_exists()? {
//         true => {
//             return Ok(format!(
//                 "{} {}\n{}",
//                 "Log location:".italic().dimmed(),
//                 &log_path.display(),
//                 fs::read_to_string(&log_path)?
//             ));
//         }
//         false => {
//             return Ok(format!(
//                 "{} {}",
//                 "No log file found:".red().bold().to_string(),
//                 log_path.display()
//             ))
//         }
//     }
// }

fn mk_bkp(source_dir: PathBuf, original_source: bool, target_dir: PathBuf) -> io::Result<()> {
    if original_source {
        let mut bkp_dir = PathBuf::new();
        let datetime = Local::now().format("%d%m%Y_%H%M%S_%f").to_string();
        match source_dir.file_name() {
            Some(name) => {
                let new_name = name.to_string_lossy() + "_" + datetime.as_str();
                bkp_dir.push(target_dir);
                bkp_dir.push(new_name.as_ref());
            }
            None => {
                error!("Unable to extract filename")
            }
        }
    }

    for entry in fs::read_dir(source_dir)? {
        let entry = entry?;
        if entry.path().is_dir() {
            todo!()
            // TODO FIXME does this work?
            //    no, every new folder gets copied directly into "bkp"
            // mkbkp(entry.path(), false, new_target_dir);
        } else if entry.path().is_file() {
            println!("Copy file: {}", entry.path().display());
            // TODO check this
            // fs::copy(entry.path(), &bkp_dir)?;
        }
    }

    Ok(())
}
