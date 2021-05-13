use cms_notifs::run;

#[tokio::main]
async fn main() -> Result<(), reqwest::Error> {
    run().await
}
