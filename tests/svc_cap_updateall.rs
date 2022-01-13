use capabilities::SqliteDb;
use capabilities_derive::capabilities;
use capabilities_derive::capability;
use capabilities_derive::service;

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
    Ok(())
}

#[capability(UpdateAll, Orders)]
fn update_orders(orders: Vec<Orders>) -> Result<Vec<Orders>, CapServiceError> {
    let data: Vec<Orders> = vec![];
    for o in orders {
        data.push(o);
    }
    Ok(data)
}
