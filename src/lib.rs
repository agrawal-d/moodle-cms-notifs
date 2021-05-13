use home;
use reqwest;
use serde::{Deserialize, Serialize};
use serde_json;
use std::error;
use std::fs;
use std::path::Path;
use web_view::*;
use webbrowser::open;
mod gui;

static CONFIG_STORE_LOCATION: &str = ".cms_notifs.json";
static DEFAULT_MOODLE_LOCATION: &str = "https://cms.bits-hyderabad.ac.in";
static NOTIFS_PARTIAL_ENDPOINT: &str = "/webservice/rest/server.php?wsfunction=message_popup_get_popup_notifications&moodlewsrestformat=json&wstoken=@&limit=100&offset=0&useridto=0";

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Config {
    pub moodle_location: String,
    pub token: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Notifications {
    pub notifications: Vec<Notification>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Notification {
    pub id: u64,
    pub subject: String,
    pub contexturl: String,
    pub timecreatedpretty: String,
}

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

    fn setup_config(base_config: Option<Config>) -> Config {
        let config;
        if let Some(got_config) = base_config {
            config = got_config
        } else {
            config = Config::get_initial_config()
        };

        let html_content = format!(
            "
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
",
            config.moodle_location, config.token
        );

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
        notifications.notifications.len(),
        config.moodle_location,
        notification_list_gen
    );

    web_view::builder()
        .title("CMS Notifications")
        .content(Content::Html(notification_list_html))
        .size(420, 800)
        .resizable(false)
        .debug(true)
        .user_data(())
        .invoke_handler(|webview, arg| {
            let (command, data) = split_once(arg);

            match command {
                "url" => {
                    webbrowser::open(data).expect("Unable to open url in browser");
                }
                "settings" => {
                    webview.exit();
                    Config::setup_config(Some(config.clone()));
                }
                _ => {
                    panic!("Unexpected command")
                }
            }

            Ok(())
        })
        .run()
        .unwrap();
}

pub async fn run() -> Result<(), reqwest::Error> {
    let duration = std::time::Duration::from_secs(60 * 15); // 15 minutes

    loop {
        let config = Config::retrieve();
        let tokenized_partial_endpoint = NOTIFS_PARTIAL_ENDPOINT.replace("@", &config.token);
        let notifs_endpoint = format!("{}{}", config.moodle_location, tokenized_partial_endpoint);
        let body = reqwest::get(&notifs_endpoint)
            .await
            .expect("HTTP request error")
            .text()
            .await
            .expect("HTTP request error");

        let notifications: Notifications =
            serde_json::from_str(&body).expect("Error processing request");

        display_notifications(notifications, &config);
        println!("Sleeping for 15 minutes...");
        std::thread::sleep(duration);
    }
}

fn split_once(in_string: &str) -> (&str, &str) {
    let mut splitter = in_string.splitn(2, ' ');
    let first = splitter.next().unwrap();
    let second = splitter.next().unwrap();
    (first, second)
}
