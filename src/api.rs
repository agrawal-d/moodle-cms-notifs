use crate::Config;
use crate::Notifications;
use reqwest;
use std::error::Error;

static NOTIFS_PARTIAL_ENDPOINT: &str = "/webservice/rest/server.php?wsfunction=message_popup_get_popup_notifications&moodlewsrestformat=json&wstoken=@&limit=100&offset=0&useridto=0";

static MARK_READ_PARTIAL_ENDPOINT: &str = "/webservice/rest/server.php?wsfunction=core_message_mark_all_notifications_as_read&moodlewsrestformat=json&wstoken=@&useridto=0&timecreatedto=0";

fn tokenize<'a>(endpoint: &'a str, config: &'a Config) -> String {
    let tokenized_partial_endpoint = endpoint.replace("@", &config.token);
    let complete_endpoint = format!("{}{}", config.moodle_location, tokenized_partial_endpoint);

    complete_endpoint
}

/// Fetch unread notifications from Moodle.
pub fn get_notifications(config: &Config) -> Result<Notifications, Box<dyn Error>> {
    let endpoint = tokenize(NOTIFS_PARTIAL_ENDPOINT, config);
    let req = reqwest::blocking::get(endpoint)?
        .text()
        .or_else(|err| Err(Box::new(err) as Box<dyn std::error::Error>))?;
    let notifications: Result<Notifications, _> =
        serde_json::from_str(&req).or_else(|err| Err(Box::new(err) as Box<dyn std::error::Error>));

    notifications
}

/// Mark all notifications are read.
pub fn mark_all_as_read(config: &Config) -> Result<String, Box<dyn Error>> {
    let endpoint = tokenize(MARK_READ_PARTIAL_ENDPOINT, config);
    let res = reqwest::blocking::get(endpoint)?
        .text()
        .or_else(|err| Err(Box::new(err) as Box<dyn std::error::Error>));

    res
}
