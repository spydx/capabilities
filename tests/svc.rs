use capabilities_derive::svc;
use sqlx::pool::Pool;
use sqlx::sqlite::Sqlite;

#[svc(Sqlite)]
#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let connection_string = "Kenneth".to_string();
    let pool = CapService::build(connection_string)
        .await
        .expect("Failed to create database");

    println!("{}", pool.con);
    Ok(())
}