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
    static LOGS_STORE_LOCATION: &str = ".cms_notifs.log"; // The location where the config is stored.
    let home_dir = home::home_dir().unwrap();
    let log_dir = home_dir.join(LOGS_STORE_LOCATION);
    let log_path_raw = log_dir.to_str().unwrap();

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
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    setup_logging();
    process_cli_args();
    info!("CMS Notifications started");
    run()
}
