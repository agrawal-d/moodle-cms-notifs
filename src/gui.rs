use std::fs;
use web_view::*;

pub fn show_notifs() {
    let html = fs::read_to_string("notifications.html").expect("Cant read");
    web_view::builder()
        .title("CMS Notifications")
        .content(Content::Html(html))
        .size(320, 480)
        .resizable(false)
        .debug(true)
        .user_data(())
        .invoke_handler(|_webview, _arg| Ok(()))
        .run()
        .unwrap();
}
