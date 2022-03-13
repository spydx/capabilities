use capabilities::SqliteDb;
use capabilities_derive::capabilities;
use capabilities_derive::capability;
use capabilities_derive::service;
use capabilities::DeleteAll;
#[capabilities(DeleteAll, id = "id")]
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

    let mut orders = vec![];
    let order = Orders { id: 1, name: "bad shit needs to be deleted".to_string()};
    orders.push(order);

    let r = match delete_all_orders(&_pool, orders, Capability::DeleteAll).await {
        Ok(_) => true,
        Err(_) => false,
    };

    assert!(r);

    Ok(())
}

#[capability(DeleteAll, Orders)]
fn delete_all_orders(_orders: Vec<Orders>) -> Result<(), CapServiceError> {
    Ok(())
}
