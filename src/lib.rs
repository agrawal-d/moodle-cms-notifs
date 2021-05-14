use crate::api::get_notifications;
use home;
use serde::{Deserialize, Serialize};
use serde_json;
use std::fs;
use std::path::Path;
use web_view::*;
use webbrowser;

mod api;
mod gui;

static CONFIG_STORE_LOCATION: &str = ".cms_notifs.json";
static DEFAULT_MOODLE_LOCATION: &str = "https://cms.bits-hyderabad.ac.in";

/// Application configuration.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Config {
    pub moodle_location: String,
    pub token: String,
}

/// The representation of the notification object returned by Moodle.
#[derive(Serialize, Deserialize, Debug)]
pub struct Notifications {
    pub notifications: Vec<Notification>,
    pub unreadcount: u32,
}

/// A notification object.
#[derive(Serialize, Deserialize, Debug)]
pub struct Notification {
    pub id: u64,
    pub subject: String,
    pub contexturl: String,
    pub timecreatedpretty: String,
}

/// Get thhe path where the config file should be saved.
fn get_config_path() -> String {
    let home_dir = home::home_dir().unwrap();
    let config_path = home_dir.join(CONFIG_STORE_LOCATION);
    let config_path_raw = config_path.to_str().unwrap();
    String::from(config_path_raw)
}

impl Config {
    /// Retrieve configuration from JSON file.
    /// If it does not exists, or is corrupted, create a new configuration.
    fn retrieve() -> Config {
        let config_exists = Path::new(&get_config_path()).exists();
        let initial_config = Config::get_initial_config();
        if !config_exists {
            Config::setup_config(Some(initial_config))
        } else {
            let config_raw =
                fs::read_to_string(&get_config_path()).expect("Unable to read config file");
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
        fs::write(&get_config_path(), serialized).expect("Failed to write configuration to disk.");
    }

    /// Open a webview to ask for config values.
    fn setup_config(base_config: Option<Config>) -> Config {
        let config;
        if let Some(got_config) = base_config {
            config = got_config
        } else {
            config = Config::get_initial_config()
        };

        let html_content = format!("
        <!doctype html>
        <html>
        <body>
        <script>
        window.external={{invoke:function(x){{window.webkit.messageHandlers.external.postMessage(x);}}}};
        function save(){{
        const moodle_location = document.getElementById('mdl-url').value;
        const token = document.getElementById('mdl-token').value;
        const config = {{moodle_location, token}};
        external.invoke(JSON.stringify(config));
        }}
        </script>
        <label>Moodle URL: <input id='mdl-url' value='{}' /></label>
        <br/>
        <label>Authentication token: <input id='mdl-token' value='{}' /></label>
        <p>You can generate authentication token by visiting CMS > Preferences > User Account > Security Keys.</p>
        <br/>
        <button onclick='save()'>Save</button>
        </body>
        </html>
        ",config.moodle_location, config.token);

        web_view::builder()
            .title("CMS Notifications Configuration")
            .content(Content::Html(html_content))
            .size(320, 480)
            .resizable(false)
            // .debug(true)
            .user_data(())
            .invoke_handler(|webview, arg| match arg {
                _ => {
                    let mut config: Config = serde_json::from_str(&arg).unwrap();
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
    let mut notification_list_gen = String::from("");

    for notif in notifications.notifications.iter() {
        let curr_notif = format!(
            "<li><p><b>{}</b> <br/><small>{}</small></p><a href='#' onclick='openurl(\"{}\")'>View</a></li>",
            notif.subject, notif.timecreatedpretty,notif.contexturl
        );
        notification_list_gen.push_str(&curr_notif);
    }

    let notification_list_html = format!(
    "<html><body>
    <script>
    window.external={{invoke:function(x){{window.webkit.messageHandlers.external.postMessage(x);}}}};
    function openurl(link){{
        external.invoke('url ' + link);
    }}

    </script>
    <h3>{} unread notifications</h3>
    <p>
    <button onclick='openurl(\"{}\")'>Open CMS</button>
    <button onclick=\"external.invoke('settings nodata')\">Settings</button>
    </p>
    <ul>{}</ul></body></html>",
        notifications.unreadcount, config.moodle_location, notification_list_gen
    );

    web_view::builder()
        .title("CMS Notifications")
        .content(Content::Html(notification_list_html))
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
pub fn display_errors(err: Box<dyn std::error::Error>) {
    let error_message = (*err).to_string();
    let html_content = format!(
        "<h1>Error</h1><pre>{}</pre><br/>The application will keep running despite the error.",
        error_message
    );
    web_view::builder()
        .title("CMS Notifications Error")
        .content(Content::Html(html_content))
        .size(400, 400)
        .resizable(false)
        .user_data(())
        .invoke_handler(|_webview, _arg| Ok(()))
        .run()
        .unwrap();
}

/// Run the application in a loop.
/// Fetches and displays notifications every 15 minutes.
pub async fn run() -> Result<(), Box<dyn std::error::Error>> {
    let duration = std::time::Duration::from_secs(60 * 15); // 15 minutes

    loop {
        let config = Config::retrieve();
        let notifs = get_notifications(&config).await;

        match notifs {
            Ok(notifs) => display_notifications(notifs, &config),
            Err(e) => display_errors(e),
        };

        println!("Sleeping for 15 minutes.");
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
