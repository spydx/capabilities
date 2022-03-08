pub extern crate capabilities_derive;
pub mod cap_http;


use actix_web::dev::ServiceRequest;
use actix_web_httpauth::extractors::bearer::BearerAuth;
pub use ::capabilities_derive::capability;
pub use ::capabilities_derive::service;

use reqwest::Client;
use sqlx::pool::Pool;
use sqlx::sqlite::Sqlite;
use sqlx::Postgres;

use actix_web::{HttpRequest, FromRequest, Error, Result};
use actix_web::HttpMessage;
use actix_web::dev::Payload;
use futures_util::future::{Ready, ok};

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


#[derive(Debug, Clone, Copy)]
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
const BASEPATH: &str = "http://localhost:8000/gnap";

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
        Ok(ir) =>  {
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
                },
                false => {
                    println!("{:#?}", ir);
                    return Err(actix_web::error::ErrorForbidden("Inactive token"))
                }
            }
        },
        Err(_) => {
            return Err(actix_web::error::ErrorForbidden("Cannot introspect this token"))
        }
    }
}

fn get_access_type(access_list: &Vec<AccessRequest>) -> Result<Vec<Capability>, Error>{
    let mut caps = vec![];
    for access in access_list {
        match access {
            AccessRequest::Value { actions, ..
                } => {
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
                },
            _ => return Err(actix_web::error::ErrorForbidden("Unknown access type")),
        }
    }
    Ok(caps)
}