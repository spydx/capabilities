use capabilities_derive::svc;
use capabilities::{PoolSqlite, WebService};
use sqlx::pool::Pool;


#[svc(PoolSqlite)]
#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let connection_string = "Kenneth".to_string();
    let pool = CapService::build(connection_string)
        .await
        .expect("Failed to create database");

    println!("{:?}", pool.con);
    Ok(())
}