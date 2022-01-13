pub extern crate capabilities_derive;

pub use ::capabilities_derive::capability;
pub use ::capabilities_derive::service;

use reqwest::Client;
use sqlx::pool::Pool;
use sqlx::sqlite::Sqlite;
use sqlx::Postgres;

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

pub type SqliteDb = Pool<Sqlite>;
pub type PostgresDb = Pool<Postgres>;
pub type WebService = Client;
pub struct EmptyInput;


use actix_web;
use actix_web::dev::ServiceRequest;
use actix_web_httpauth::extractors::bearer::BearerAuth;

pub async fn token_validator(
    req: ServiceRequest,
    _header: BearerAuth,
) -> Result<ServiceRequest, actix_web::Error> {
    println!("{}", _header.token());
    Ok(req)
}
