use capabilities::EmptyInput;
use capabilities::SqliteDb;
use capabilities_derive::capabilities;
use capabilities_derive::capability;
use capabilities_derive::service;

#[capabilities(ReadAll, id = "id")]
pub struct Orders {
    #[allow(dead_code)]
    id: i32,
    #[allow(dead_code)]
    name: String,
}

#[service(SqliteDb, name = "db")]
#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let connection_string = "sqlite::memory:".to_string();
    let _pool = CapService::build(connection_string)
        .await
        .expect("Failed to create database");
    Ok(())
}

// if arg are missing, we get a unwrap error
// Capability<ReadAll<Orders>> this is a problem
#[capability(ReadAll, Orders)]
fn get_order(id: i32) -> Result<Vec<Orders>, CapServiceError> {
    let data: Vec<Orders> = vec![];
    Ok(data)
}
