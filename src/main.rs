//! Read your latest unread notifications from Moodle.

#[macro_use]
extern crate log;
extern crate simplelog;

use cms_notifs;
use cms_notifs::run;
use home;
use simplelog::*;
use std::collections::HashSet;
use std::env;
use std::fs::File;

// Process the command line arguments.
fn process_cli_args() {
    let args: HashSet<String> = env::args().collect();

    if args.contains("--silent-errors") {
        info!("Errors will not show up in an error window. ( Reason: \"--silent-errors\" argument passed. )");
    }

    if args.contains("--settings") {
        cms_notifs::Config::setup_config(Some(cms_notifs::Config::retrieve()));
        std::process::exit(0);
    } else {
        info!("Note: You can run with --settings argument to open the settings dialog.");
    }
}

// Configure logging to a file.
fn setup_logging() {

    let home_dir = home::home_dir().unwrap();

    let log_dir = match env::consts::OS {
        "linux" => match env::var_os("XDG_STATE_HOME") {
            Some(dir) => std::path::PathBuf::from(&dir),
            None => home_dir.join(".local").join("state"),
        },
        _ => home_dir,
    };
    std::fs::create_dir_all(&log_dir).unwrap();
    let log_path = match env::consts::OS {
        "linux" => log_dir.join("cms_notifs.log"),
        _ => log_dir.join(".cms_notifs.log"),
    };

    let log_path_raw = log_path.to_str().unwrap();
    CombinedLogger::init(vec![
        TermLogger::new(
            LevelFilter::Info,
            Config::default(),
            TerminalMode::Mixed,
            ColorChoice::Auto,
        ),
        WriteLogger::new(
            LevelFilter::Info,
            Config::default(),
            File::create(log_path_raw).unwrap(),
        ),
    ])
    .unwrap();
    info!("Storing logs at {}", log_path_raw);
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    setup_logging();
    process_cli_args();
    info!("CMS Notifications started");
    run()
}
