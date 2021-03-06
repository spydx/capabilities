use capabilities::SqliteDb;
use capabilities_derive::capabilities;
use capabilities_derive::capability;
use capabilities_derive::service;
use capabilities::UpdateAll;

#[capabilities(UpdateAll, id = "id")]
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
    let mut list_of_orders = vec![];
    let o = Orders { id: 123123, name: "Secret order".to_string()};
    list_of_orders.push(o);

    let r = match update_orders(&_pool, list_of_orders, Capability::UpdateAll).await {
        Ok(_) => true,
        Err(_) => false,
    };

    assert!(r);
    Ok(())
}

#[capability(UpdateAll, Orders)]
fn update_orders(orders: Vec<Orders>) -> Result<(), CapServiceError> {
    let mut data: Vec<Orders> = vec![];
    for o in orders {
        data.push(o);
    }
    Ok(())
}
