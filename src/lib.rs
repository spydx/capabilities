pub extern crate capabilities_derive;

pub use ::capabilities_derive::capability;
pub use ::capabilities_derive::service;

use reqwest::Client;
use sqlx::pool::Pool;
use sqlx::sqlite::Sqlite;
use sqlx::Postgres;

pub enum Operations {
    Create,
    Read,
    Update,
    Delete,
    ReadAll,
    UpdateAll,
    DeleteAll,
}

#[allow(dead_code)]
pub struct Create<T> {
    pub data: T,
}
#[allow(dead_code)]
pub struct Read<T> {
    pub data: T,
}
#[allow(dead_code)]
pub struct Update<T> {
    pub data: T,
}
#[allow(dead_code)]
pub struct Delete<T> {
    pub data: T,
}

pub struct ReadAll<T> {
    pub data: T,
}
pub struct UpdateAll<T> {
    pub data: T,
}
pub struct DeleteAll<T> {
    pub data: T,
}

pub type PoolSqlite = Pool<Sqlite>;
pub type PoolPostgres = Pool<Postgres>;
pub type WebService = Client;
