use capabilities::SqliteDb;
use capabilities_derive::capabilities;
use capabilities_derive::capability;
use capabilities_derive::service;
use capabilities::Create;

#[capabilities(Create, id = "id")]
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

    let order = Orders { id: 1, name: "Expensive stuff".to_string()};

    let r = match create_order(&_pool, order, Capability::Create).await {
        Ok(d) => Some(d),
        Err(_) => None,
    };

    assert!(r.is_some());
    assert_eq!(r.unwrap().id, 1);

    Ok(())
}

#[capability(Create, Orders)]
fn create_order(order: Orders) -> Result<Orders, CapServiceError> {
    
    Ok(Orders {
        id: order.id,
        name: order.name,
    })
}