use capabilities::WebService;
use capabilities::WebService;
use capabilities_derive::svc;

#[svc(WebService)]
#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let base_url = "https://api.kefo.no/".to_string();

    let api = CapService::build(base_url)
        .await
        .expect("Failed to setup service");

    println!("{:?}", api.con);
    Ok(())
}
