use capabilities::PoolSqlite;
use capabilities_derive::service;
use capabilities_derive::capabilities;
use sqlx::Pool;


#[capabilities(Read, Update)]
struct Orders {
    id: i32, 
    name: String, 
}

#[service(PoolSqlite)]
#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let connection_string = "sqlite::memory:".to_string();
    let _pool = CapService::build(connection_string)
        .await
        .expect("Failed to create database");
    Ok(())
}
