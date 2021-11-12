use capabilities::PoolSqlite;
use capabilities_derive::capabilities;
use capabilities_derive::capability;
use capabilities_derive::service;
use sqlx::Pool;

#[capabilities(Read)]
pub struct Orders {
    #[allow(dead_code)]
    id: i32,
    #[allow(dead_code)]
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

#[capability(Read, "Orders")]
fn get_order(order: Orders) -> Result<Order, CapServiceError> {
    Ok(Orders { id: 1, name: "MY order".to_string(),})
}