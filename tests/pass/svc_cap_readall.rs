use capabilities::SqliteDb;
use capabilities_derive::capabilities;
use capabilities_derive::capability;
use capabilities_derive::service;
use capabilities::ReadAll;

#[capabilities(ReadAll, id = "id")]
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

    let r = match get_orders(&_pool, Capability::ReadAll).await {
        Ok(d) => Some(d),
        Err(_) => None,
    };

    assert!(r.is_some());
    
    Ok(())
}

#[capability(ReadAll, Orders)]
fn get_orders() -> Result<Vec<Orders>, CapServiceError> {
    let data: Vec<Orders> = vec![];
    Ok(data)
}
