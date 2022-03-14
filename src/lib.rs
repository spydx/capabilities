pub extern crate capabilities_derive;

pub use ::capabilities_derive::capability;
pub use ::capabilities_derive::service;
use actix_web::dev::ServiceRequest;
use actix_web_httpauth::extractors::bearer::BearerAuth;

use reqwest::Client;
use sqlx::pool::Pool;
use sqlx::sqlite::Sqlite;
use sqlx::Postgres;

use actix_web::dev::Payload;
use actix_web::HttpMessage;
use actix_web::{Error, FromRequest, HttpRequest, Result};
use futures_util::future::{ok, Ready};

use gnap_cli::introspect;
use gnap_cli::models::access_token::AccessRequest;
use log::debug;

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Capability {
    Read,
    ReadAll,
    Write,
    Create,
    Update,
    UpdateAll,
    Delete,
    DeleteAll,
    Invalid,
}

impl FromRequest for Capability {
    type Error = Error;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _payload: &mut Payload) -> Self::Future {
        let c = req.extensions().get::<Capability>().unwrap().clone();
        println!("From middleware: {:#?}", c);
        ok(c)
    }
}

struct User;

pub trait CapToEnum {
    fn into_enum(&self) -> Capability;
}

impl CapToEnum for Read<User> {
    fn into_enum(&self) -> Capability {
        Capability::Read
    }
}


const BASEPATH: &str = "http://localhost:8000/gnap";


#[derive(Clone)]
pub struct FilterConfig {
    pub basepath: String,
    pub rs_ref: String,
}


impl FilterConfig {
    pub fn build(basepath: String, rs_ref: String) -> Self {
        Self {
            basepath: basepath,
            rs_ref: rs_ref
        }
    }
}


pub async fn token_introspection(
    req: ServiceRequest,
    header: BearerAuth,
) -> Result<ServiceRequest, Error> {
    debug!("Token: {}", header.token());
    debug!("{:#?}", req);

    let token = header.token().to_string();
    println!("{:#?}", token);
    let rs_ref = "e8a2968a-f183-45a3-b63d-4bbbd1dad276".to_string();
    let url = format!("{}", BASEPATH);

    match introspect(token, rs_ref, url).await {
        Ok(ir) => {
            match ir.active {
                true => {
                    //debug!("{:#?}", ir);
                    let access_req = ir.access.unwrap();
                    let cap = match get_access_type(&access_req) {
                        Ok(cap) => cap.first().unwrap().to_owned(),
                        Err(_) => Capability::Invalid,
                    };
                    req.extensions_mut().insert(cap);
                    println!("{:#?}", req);
                    Ok(req)
                }
                false => {
                    println!("{:#?}", ir);
                    return Err(actix_web::error::ErrorForbidden("Inactive token"));
                }
            }
        }
        Err(_) => {
            return Err(actix_web::error::ErrorForbidden(
                "Cannot introspect this token",
            ))
        }
    }
}

fn get_access_type(access_list: &Vec<AccessRequest>) -> Result<Vec<Capability>, Error> {
    let mut caps = vec![];
    for access in access_list {
        match access {
            AccessRequest::Value { actions, .. } => {
                for action in actions.clone().unwrap() {
                    match action.as_str() {
                        "read" => caps.push(Capability::Read),
                        "create" => caps.push(Capability::Create),
                        "write" => caps.push(Capability::Write),
                        "update" => caps.push(Capability::Update),
                        "delete" => caps.push(Capability::Delete),
                        _ => caps.push(Capability::Invalid),
                    }
                }
            }
            _ => return Err(actix_web::error::ErrorForbidden("Unknown access type")),
        }
    }
    Ok(caps)
}


#[cfg(test)]
mod test {
    use super::*;
    
    #[test]
    fn compare_to_enums() {

        let res = Capability::Read.eq(&Capability::Read);
        assert!(res)
    }

    
    #[allow(dead_code)]
    struct User {
        pub name: String,
    }
    impl CapToEnum for Read<User> {
        fn into_enum(&self) -> Capability {
            Capability::Read
        }
    }

    #[test]
    fn convert_struct_to_enum() {
        let user = User { name: "Kenneth".to_string()};
        let read_user = crate::Read::<User> { data: user };
        let c = read_user.into_enum();
        assert_eq!(c, Capability::Read);
        assert_ne!(c, Capability::Delete);

    }
}