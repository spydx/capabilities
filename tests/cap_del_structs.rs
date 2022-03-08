use core::fmt;

use capabilities::Delete;
use capabilities::{capability, SqliteDb};
use capabilities_derive::capabilities;
use capabilities_derive::service;

#[service(SqliteDb)]
#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let connection_string = "sqlite::memory:".to_string();
    let pool = CapService::build(connection_string)
        .await
        .expect("Failed to create database");

    // Should be a DTO and not a "Person"
    let temp: Person = Person {
        personnummer: 0,
        firstname: "Kenenth".to_string(),
        lastname: "Fossen".to_string(),
    };

    match delete_a_person(&pool, temp).await {
            Ok(_) => println!("Deleted"),
            Err(_) => println!("Unexpected error")
    }

    Ok(())
}

#[capabilities(Delete, id = "personnummer")]
#[derive(Debug)]
#[allow(dead_code)]
pub struct Person {
    personnummer: i64,
    firstname: String,
    lastname: String,
}

impl fmt::Display for Person {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {})", self.firstname, self.lastname)
    }
}

#[capability(Delete, Person)]
pub fn delete_a_person(_data: Person) -> Result<(), CapServiceError> {
    Ok(())
}

#[capability(Delete, Person, id = "i64")]
pub fn delete_a_person_by_id(_data: PersonId) -> Result<(), CapServiceError> {
    Ok(())
}
