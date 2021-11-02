use capabilities_derive::service;

#[service]
struct Database {
    pub db: String,
}

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let name = "Kenneth".to_string();
    let pool = Database::build(name)
        .await
        .expect("Failed to create database");

    println!("{}", pool.db);
    Ok(())
}
