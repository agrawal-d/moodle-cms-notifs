//! Read your latest unread notifications from Moodle.

use cms_notifs::run;
use cms_notifs::Config;
use std::env;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();

    if args.len() == 2 && args[1] == "--settings" {
        Config::setup_config(Some(Config::retrieve()));
        std::process::exit(0);
    } else {
        println!("Note: You can run with --settings argument to open the settings dialog.");
    }

    run()
}
