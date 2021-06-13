#[macro_use]
extern crate log;
extern crate simplelog;

use crate::api::get_notifications;
use home;
use serde::{Deserialize, Serialize};
use serde_json;
use std::collections::HashSet;
use std::env;
use std::fs;
use std::path::Path;
use web_view::*;
use webbrowser;

mod api;
mod html;

static DEFAULT_MOODLE_LOCATION: &str = "https://cms.bits-hyderabad.ac.in"; // The autofilled Moodle endpoint.

/// Application configuration format.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Config {
    pub moodle_location: String,
    pub token: String,
}

struct LaunchConfig {
    pub silent_errors: bool,
}

#[macro_use]
extern crate lazy_static;

// Store relevant CLI args as global statics for later use.
lazy_static! {
    static ref LAUNCH_CONFIG: LaunchConfig = {
        let args: HashSet<String> = env::args().collect();
        LaunchConfig {
            silent_errors: args.contains("--silent-errors"),
        }
    };
}

/// The representation of the notification objects returned by Moodle.
#[derive(Serialize, Deserialize, Debug)]
pub struct Notifications {
    pub notifications: Vec<Notification>,
    pub unreadcount: usize,
}

/// A notification object.
#[derive(Serialize, Deserialize, Debug)]
pub struct Notification {
    pub id: u64,
    pub subject: String,
    pub contexturl: Option<String>,
    pub useridto: u32,
    pub text: Option<String>,
    pub timecreatedpretty: String,
}

impl Config {
    /// Retrieve configuration from JSON file.
    /// If it does not exist, or is corrupted, create a new configuration.
    pub fn retrieve() -> Config {
        let config_exists = Path::new(&Config::get_config_path()).exists();
        let initial_config = Config::get_initial_config();
        if !config_exists {
            Config::setup_config(Some(initial_config))
        } else {
            let config_raw =
                fs::read_to_string(&Config::get_config_path()).expect("Unable to read config file");
            if let Ok(config) = serde_json::from_str(&config_raw) {
                config
            } else {
                Config::store(&initial_config);
                Config::setup_config(Some(initial_config))
            }
        }
    }

    /// Get a config value with sane defaults where possible.
    fn get_initial_config() -> Config {
        Config {
            moodle_location: String::from(DEFAULT_MOODLE_LOCATION),
            token: String::from(""),
        }
    }

    /// Store config to disk.
    fn store(config: &Config) {
        let serialized = serde_json::to_string(config).expect("Unable to serialize config.");
        fs::write(&Config::get_config_path(), serialized)
            .expect("Failed to write configuration to disk.");
    }

    /// Get thhe path where the config file should be saved.
    fn get_config_path() -> String {
        let home_dir = home::home_dir().unwrap();

        let config_dir = match env::consts::OS {
            "linux" => match env::var_os("XDG_CONFIG_HOME") {
                Some(dir) => std::path::PathBuf::from(&dir),
                None => home_dir.join(".config"),
            },
            _ => home_dir,
        };
        std::fs::create_dir_all(&config_dir).unwrap();
        let config_path = match env::consts::OS {
            "linux" => config_dir.join("cms_notifs.json"),
            _ => config_dir.join(".cms_notifs.json")
        };

        let config_path_raw = config_path.to_str().unwrap();
        String::from(config_path_raw)
    }

    /// Open a webview to ask for config values.
    pub fn setup_config(base_config: Option<Config>) -> Config {
        let config;
        if let Some(got_config) = base_config {
            config = got_config
        } else {
            config = Config::get_initial_config()
        };

        let html_stub = format!(
            "
            <script>
            function save(){{
                const moodle_location = document.getElementById('mdl-url').value;
                const token = document.getElementById('mdl-token').value;
                const config = {{moodle_location, token}};
                sendMessage('config', JSON.stringify(config));
            }}
            </script>
            <label>Moodle URL:<br/><input id='mdl-url' value='{}' /></label>
            <br/>
            <label>Authentication token:<br/><input id='mdl-token' value='{}' /></label>
            <br/>
            <small>You can generate authentication token by visiting CMS > Preferences > User Account > Security Keys. Use the 'Moodle mobile web service' token.</small>
            <br/>
            <button onclick='save()'>Save</button>
        ",config.moodle_location, config.token);

        let html_content = html::complete_html(&html_stub);

        web_view::builder()
            .title("CMS Notifications Configuration")
            .content(Content::Html(html_content))
            .size(320, 480)
            .resizable(false)
            // .debug(true)
            .user_data(())
            .invoke_handler(|webview, arg| match arg {
                _ => {
                    let (_command, data) = split_once(arg);
                    let mut config: Config = serde_json::from_str(data).unwrap();
                    if config.moodle_location.ends_with('/') {
                        config.moodle_location.pop();
                    }
                    Config::store(&config);
                    webview.exit();
                    Ok(())
                }
            })
            .run()
            .unwrap();

        Config::retrieve()
    }
}

/// Open a webview to show the notifications.
pub fn display_notifications(notifications: Notifications, config: &Config) {
    if notifications.unreadcount == 0 {
        info!("0 unread notifications");
        return;
    }

    let mut notification_list_gen = String::from("");
    let my_user_id: u32 = match notifications.notifications.get(0) {
        Some(notif) => notif.useridto,
        None => 0,
    };

    for notif in notifications.notifications.iter() {
        let url = if let Some(link) = &notif.contexturl {
            link
        } else {
            &config.moodle_location
        };

        let details = if let Some(text) = &notif.text {
            text
        } else {
            "Open to view more details"
        };

        let curr_notif = format!(
            "<li><details><summary><b>{}</b><br/><small>{}</small></summary><p>{}<p></details><a href='#' onclick=\"sendMessage('url','{}')\">View</a></li>",
            notif.subject, notif.timecreatedpretty,details, url
        );
        notification_list_gen.push_str(&curr_notif);
    }

    let notification_list_html_stub = format!(
        "
        <h3>{} unread notifications</h3>
        <p>
        <button onclick=\"sendMessage('mark_read')\">Mark as read</button>
        <button onclick=\"sendMessage('url','{}')\">Open CMS</button>
        <button onclick=\"sendMessage('settings')\">Settings</button>
        </p>
        <ul>{}</ul></body></html>",
        notifications.unreadcount, config.moodle_location, notification_list_gen
    );

    let html_content = html::complete_html(&notification_list_html_stub);

    web_view::builder()
        .title("CMS Notifications")
        .content(Content::Html(&html_content))
        .size(420, 800)
        .resizable(false)
        .user_data(())
        .invoke_handler(|_webview, arg| {
            let (command, data) = split_once(arg);

            match command {
                "url" => {
                    webbrowser::open(data).expect("Unable to open url in browser");
                }
                "settings" => {
                    Config::setup_config(Some(config.clone()));
                }
                "mark_read" => {
                    api::mark_all_as_read(config, my_user_id)
                        .expect("Could not mark notifications as read");
                }
                other => {
                    eprintln!("Unexpected command from webview {}", other);
                }
            }

            Ok(())
        })
        .run()
        .unwrap();
}

/// Open a webview to show an error message.
pub fn display_errors(config: &Config, err: Box<dyn std::error::Error>) {
    let error_message = (*err).to_string();
    error!("{}", error_message);

    if crate::LAUNCH_CONFIG.silent_errors {
        return;
    }

    let html_stub = format!(
        "
        <h1>Error</h1>
        <button onclick=\"sendMessage('settings')\">Settings</button>
        <pre>{}</pre><br/>Errors can happen if you provided an invalid authentication token, or if Moodle is unreachable.",
        error_message
    );
    let html_content = html::complete_html(&html_stub);
    web_view::builder()
        .title("CMS Notifications Error")
        .content(Content::Html(html_content))
        .size(400, 400)
        .resizable(false)
        .user_data(())
        .invoke_handler(|_webview, arg| {
            let (command, _data) = split_once(arg);
            match command {
                "settings" => {
                    Config::setup_config(Some(config.clone()));
                }
                other => {
                    eprintln!("Unexpected command from webview {}", other);
                }
            }
            Ok(())
        })
        .run()
        .unwrap();
}

/// Run the application in a loop.
/// Fetche and display notifications every 15 minutes.
pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    let duration = std::time::Duration::from_secs(60 * 15); // 15 minutes

    loop {
        {
            let config = Config::retrieve();
            let notifs = get_notifications(&config);

            match notifs {
                Ok(notifs) => {
                    display_notifications(notifs, &config);
                }
                Err(e) => {
                    display_errors(&config, e);
                }
            };
        }

        info!("Sleeping for 15 minutes.");
        std::thread::sleep(duration);
    }
}

/// Split a string into two on first whitespace.
fn split_once(in_string: &str) -> (&str, &str) {
    let mut splitter = in_string.splitn(2, ' ');
    let first = splitter.next().unwrap();
    let second = splitter.next().unwrap();
    (first, second)
}
