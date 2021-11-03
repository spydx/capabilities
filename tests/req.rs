use capabilities::WebService;
use capabilities_derive::svc;
use reqwest::Client;

#[svc(WebService)]
#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let api = CapService::build()
        .await
        .expect("Failed to setup service");

    println!("{:?}", api.con);
    Ok(())
}
