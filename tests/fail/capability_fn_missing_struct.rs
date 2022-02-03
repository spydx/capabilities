use capabilities::SqliteDb;
use capabilities_derive::capabilities;
use capabilities_derive::capabilty;
use capabilities_derive::service;
use capabilities::*;


struct Orders {
    id: i32,
    name: String,
}

#[service(SqliteDb)]
#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let connection_string = "sqlite::memory:".to_string();
    let _pool = CapService::build(connection_string)
        .await
        .expect("Failed to create database");
    Ok(())
}


#[capabilty(Orders, Read)]
fn get_order() -> Result<Orders, CapServiceErrro> {
    Ok(Orders {
        id: 1,
        name: String::from("lkajsdlkf"),
    })
}