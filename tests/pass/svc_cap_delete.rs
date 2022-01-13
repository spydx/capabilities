use capabilities::SqliteDb;
use capabilities_derive::capabilities;
use capabilities_derive::capability;
use capabilities_derive::service;

#[capabilities(Delete, id = "id")]
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

#[capability(Delete, Orders, id = "i32")]
fn delete_order_by_id(order_id: i32) -> Result<Orders, CapServiceError> {
    let deleted = Orders {
        id: order_id,
        name: "MY order".to_string(),
    };
    Ok(deleted)
}

#[capability(Delete, Orders)]
fn delete_order(order: Orders) -> Result<Orders, CapServiceError> {
    let _o = Orders {
        id: order.id,
        name: "MY order".to_string(),
    };
    Ok(_o)
}