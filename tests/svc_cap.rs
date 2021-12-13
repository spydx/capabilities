use capabilities::PoolSqlite;
use capabilities_derive::capabilities;
use capabilities_derive::capability;
use capabilities_derive::service;
use sqlx::Pool;

#[capabilities(Read, id = "id")]
pub struct Orders {
    #[allow(dead_code)]
    id: i32,
    #[allow(dead_code)]
    name: String,
}

// name for db field
#[service(PoolSqlite)]
#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let connection_string = "sqlite::memory:".to_string();
    let _pool = CapService::build(connection_string)
        .await
        .expect("Failed to create database");
    Ok(())
}


// The trait for CanReadOrders -> is Read<Orders> and not Read<i32>
#[capability(Read, Orders)]
fn get_order(order_id: i32) -> Result<Orders, CapServiceError> {
    Ok(Orders {
        id: 1,
        name: "MY order".to_string(),
    })
}
