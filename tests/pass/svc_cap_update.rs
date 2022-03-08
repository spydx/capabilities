use capabilities::SqliteDb;
use capabilities_derive::capabilities;
use capabilities_derive::capability;
use capabilities_derive::service;
use capabilities::Update;

#[capabilities(Update, id = "id")]
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

#[capability(Update, Orders, id = "i32")]
fn update_order_by_id(order_id: OrdersId) -> Result<(), CapServiceError> {
    Ok(Orders {
        id: order_id.id,
        name: "MY order".to_string(),
    })
}

#[capability(Update, Orders)]
fn update_order(order: Orders) -> Result<(), CapServiceError> {
    Ok(
        Orders {
            id: order.id+1,
            name: "Want more of thhis order".to_string(),
        }
    )
}