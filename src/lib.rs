pub extern crate capabilities_derive;

pub use ::capabilities_derive::capability;
pub use ::capabilities_derive::service;

use reqwest::Client;
use sqlx::Postgres;
use sqlx::pool::Pool;
use sqlx::sqlite::Sqlite;

pub enum Operations {
    Create,
    Read,
    Update,
    Delete,
    CreateAll,
    ReadAll,
    UpdateAll,
    DeleteAll,
}

pub type PoolSqlite = Pool<Sqlite>;
pub type PoolPostgres = Pool<Postgres>;
pub type WebService = Client;
