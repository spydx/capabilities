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

    let up_order = Orders { id: 1337, name: "Updated order".to_string()};

    let r = match update_order(&_pool, up_order, Capability::Update).await {
        Ok(_) => true,
        Err(_) => false,
    };

    assert!(r);
    
    let r2 = match update_order_by_id(&_pool, OrdersId{ id: 666}, Capability::Update).await {
        Ok(_) => true,
        Err(_) => false,
    };

    assert!(r2);
    Ok(())
}

#[capability(Update, Orders, id = "i32")]
fn update_order_by_id(_order_id: OrdersId) -> Result<(), CapServiceError> {
    
    Ok(())
}

#[capability(Update, Orders)]
fn update_order(_order: Orders) -> Result<(), CapServiceError> {
    Ok(())
}