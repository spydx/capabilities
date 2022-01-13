use capabilities::SqliteDb;
use capabilities_derive::capabilities;
use capabilities_derive::capability;
use capabilities_derive::service;

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
    Ok(())
}

#[capability(DeleteAll, Orders)]
fn delete_all_orders(orders: Vec<Orders>) -> Result<Vec<Orders>, CapServiceError> {
    let mut deleted_orders: Vec<Orders> = vec![];
    for o in orders {
        /* 
        sqlx::query!(r#"DELETE FROM orders WHERE id = $1 AND name = $2"#, 
            o.id,
            o.name
        ).execute(self.db)
        .await
        .expect("Failed to delete items");
        */
        deleted_orders.push(o);
    }
    Ok(deleted_orders)
}
