use crate::Config;
use crate::Notifications;
use reqwest;
use std::error::Error;
static NOTIFS_PARTIAL_ENDPOINT: &str = "/webservice/rest/server.php?wsfunction=message_popup_get_popup_notifications&moodlewsrestformat=json&wstoken=@&limit=100&offset=0&useridto=0";

/// Fetch unread notifications from Moodle.
pub async fn get_notifications(config: &Config) -> Result<Notifications, Box<dyn Error>> {
    let tokenized_partial_endpoint = NOTIFS_PARTIAL_ENDPOINT.replace("@", &config.token);
    let notifs_endpoint = format!("{}{}", config.moodle_location, tokenized_partial_endpoint);
    let req = reqwest::get(&notifs_endpoint).await?.text().await?;
    let notifications: Result<Notifications, _> =
        serde_json::from_str(&req).or_else(|err| Err(Box::new(err) as Box<dyn std::error::Error>));

    notifications
}
