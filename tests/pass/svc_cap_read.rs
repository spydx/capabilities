use capabilities::SqliteDb;
use capabilities_derive::capabilities;
use capabilities_derive::capability;
use capabilities_derive::service;
use capabilities::Read;

#[capabilities(Read, id = "id")]
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
    let pool = CapService::build(connection_string)
        .await
        .expect("Failed to create database");

    let order = Orders { id: 1, name: "All my expensive stuff".to_string()};

    let r = match read_order(&pool, order, Capability::Read).await {
        Ok(d) => Some(d),
        Err(_) => None,
    };

    assert!(r.is_some());

    let order_id = OrdersId { id :1 };
    let r2 = match read_order_by_id(&pool, order_id , Capability::Read).await {
        Ok(d) => Some(d),
        Err(_) => None,
    };
    assert!(r2.is_some());
    assert_eq!(r2.unwrap().id, 1);

    Ok(())
}


#[capability(Read, Orders)]
fn read_order(order: Orders) -> Result<Orders, CapServiceError> {
    Ok(
        Orders {
            id: order.id,
            name: order.name,
        }
    )
}
#[capability(Read, Orders, id = "i32")]
fn read_order_by_id(order_id: OrdersId) -> Result<Orders, CapServiceError> {
    Ok(Orders {
        id: order_id.id,
        name: "MY order".to_string(),
    })
}
