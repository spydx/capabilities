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

    let person: Person = delete_a_person(&pool, temp)
        .await
        .expect("Failed to create");

    println!("{person}");
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
pub fn delete_a_person(data: Person) -> Result<Person, CapServiceError> {
    Ok(Person {
        personnummer: 98127918273912,
        firstname: data.firstname,
        lastname: data.lastname,
    })
}


#[capability(Delete, Person, id = "i64")]
pub fn delete_a_person_by_id(_data: PersonId) -> Result<Person, CapServiceError> {
    Ok(Person {
        personnummer: _data,
        firstname: "fistname".to_string(),
        lastname: "lastname".to_string(),
    })
}