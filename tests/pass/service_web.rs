use capabilities::WebService;
use capabilities_derive::service;
use reqwest::Client;

#[service(WebService)]
#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let _api = CapService::build().await.expect("Failed to setup service");

    Ok(())
}
