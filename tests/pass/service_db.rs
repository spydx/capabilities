use capabilities::SqliteDb;
use capabilities_derive::service;

#[service(SqliteDb)]
#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let connection_string = "sqlite::memory:".to_string();
    let _pool = CapService::build(connection_string)
        .await
        .expect("Failed to create database");
    Ok(())
}