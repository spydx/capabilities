use capabilities::SqliteDb;
use capabilities_derive::capabilities;
use capabilities_derive::capability;
use capabilities_derive::service;
use capabilities::Delete;

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

    let order = Orders { id: 1, name: "My bad order".to_string()};

    let r = match delete_order(&_pool, order, Capability::Delete).await {
        Ok(_) => true,
        Err(_) => false,
    };

    assert!(r);

    let r2 = match delete_order(&pool, OrdersId { id: 2}, Capability::Delete).await {
        Ok(_) => true,
        Err(_) => false,
    };

    assert!(r2);

    Ok(())
}

#[capability(Delete, Orders, id = "i32")]
fn delete_order_by_id(order_id: OrdersId) -> Result<(), CapServiceError> {
    let _deleted = Orders {
        id: order_id.id,
        name: "MY order".to_string(),
    };
    Ok(())
}

#[capability(Delete, Orders)]
fn delete_order(order: Orders) -> Result<(), CapServiceError> {
    let _o = Orders {
        id: order.id,
        name: "MY order".to_string(),
    };
    Ok(())
}