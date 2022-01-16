use capabilities::SqliteDb;
use capabilities_derive::service;

#[service(SqliteDb, name = "megakult")]
#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let connection_string = "sqlite::memory:".to_string();
    let pool = CapService::build(connection_string)
        .await
        .expect("Failed to create database");
    let _clone = pool.clone();
    Ok(())
}
