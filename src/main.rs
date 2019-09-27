#[macro_use]
extern crate serde_derive;

use std::collections::HashMap;
use std::env::args;
use std::error::Error;
use std::fmt;
use std::fs::File;
use std::net::SocketAddr;
use std::str::FromStr;
use std::sync::{Arc, RwLock};

use futures::{future, Future, Stream};
use futures::future::{Either, FutureResult};
use hyper::{Method, Request, Response, rt, Server, StatusCode};
use hyper::body::Body;
use hyper::http::response::Builder;
use hyper::service::{make_service_fn, Service, service_fn_ok};
use log::{error, warn};
use r2d2_mongodb::ConnectionOptions;

use crate::error::{ErrorInfo, Never};
use crate::http::{empty_response, json_builder, json_ok};
use crate::storage::{Secret, Storage};

mod http;
mod error;
mod storage;

static DEFAULT_BIND: &str = "127.0.0.1:9000";
static CONFIG_FILE: &str = "config.json";

#[derive(Deserialize)]
pub struct DbConfig {
    host: String,
    port: u16,
    dbname: String,
    username: Option<String>,
    password: Option<String>,
    pool_size: u32,
}

fn main() {
    let address = {
        let addr = args().nth(1).unwrap_or_else(|| {
            eprintln!("No binding given, using default {}", DEFAULT_BIND);
            String::from(DEFAULT_BIND)
        });
        SocketAddr::from_str(&addr).or_else(|err| {
            eprintln!("Invalid binding given ({}), using default {}", err, DEFAULT_BIND);
            SocketAddr::from_str(DEFAULT_BIND)
        }).unwrap()
    };

    let db_conf: DbConfig = {
        let file = File::open(CONFIG_FILE).expect("Config file not found");
        serde_json::from_reader(file).expect("Invalid config format")
    };

    let conn_mgr = r2d2_mongodb::MongodbConnectionManager::new({
        let mut opts = r2d2_mongodb::ConnectionOptions::builder();
        opts.with_host(&db_conf.host, db_conf.port)
            .with_db(&db_conf.dbname);
        if let (Some(username), Some(password)) = (db_conf.username, db_conf.password) {
            opts.with_auth(&username, &password);
        }
        opts.build()
    });
    let pool = r2d2::Pool::builder()
        .max_size(db_conf.pool_size)
        .build(conn_mgr)
        .expect("Pool connection error");

    let storage = Arc::new(RwLock::new(HashMap::new()));

    let server = Server::bind(&address)
        .serve(make_service_fn(move |_| future::ok::<_, Never>(SecretService::new(storage.clone()))))
        .map_err(|e| error!("Error: {:?}", e));

    rt::run(rt::lazy(move || {
        rt::spawn(server);
        Ok(())
    }));
}

pub struct SecretService<S> {
    storage: S,
}

impl<S> SecretService<S> {
    fn new(storage: S) -> Self {
        SecretService { storage }
    }
}

impl Default for SecretService<Arc<RwLock<HashMap<String, Secret>>>> {
    fn default() -> Self {
        SecretService::new(Arc::new(RwLock::new(HashMap::new())))
    }
}

impl<S: Storage + Clone + 'static> Service for SecretService<S> {
    type ReqBody = Body;
    type ResBody = Body;
    type Error = Never;
    type Future = Either<FutureResult<Response<Body>, Never>, Box<dyn Future<Item=Response<Body>, Error=Never> + Send>>;

    fn call(&mut self, req: Request<Self::ReqBody>) -> Self::Future {
        let method = req.method();
        let mut path = req.uri().path().trim_start_matches("/").split("/");
        let domain = match path.next() {
            None => return Either::A(ErrorInfo::new("Domain name missing").resp(StatusCode::BAD_REQUEST)),
            Some(domain) => domain.to_string(),
        };
        match method {
            &Method::GET if domain.is_empty() =>
                Either::A(match self.storage.get_all() {
                    Ok(values) => json_ok(&values),
                    Err(err) => ErrorInfo::new(&format!("Storage error: {:?}", err))
                        .resp(StatusCode::INTERNAL_SERVER_ERROR),
                }),
            &Method::GET =>
                Either::A(match self.storage.get(&domain) {
                    Ok(Some(secret)) => json_ok(&secret),
                    Ok(None) => ErrorInfo::new("Domain not found").resp(StatusCode::NOT_FOUND),
                    Err(err) => ErrorInfo::new(&format!("Storage error: {:?}", err))
                        .resp(StatusCode::INTERNAL_SERVER_ERROR),
                }),
            &Method::PUT | &Method::POST => {
                let storage = self.storage.clone();
                let resp = req.into_body()
                    .map_err(|err| {
                        panic!("Error processing request: {}", err);
                    })
                    .concat2()
                    .and_then(move |c| {
                        match String::from_utf8(c.to_vec()) {
                            Err(err) =>
                                ErrorInfo::new(&format!("invalid data: {}", err))
                                    .resp(StatusCode::BAD_REQUEST),
                            Ok(s) => match serde_json::from_str::<Secret>(&s) {
                                Err(err) =>
                                    ErrorInfo::new(&format!("invalid json: {}", err))
                                        .resp(StatusCode::BAD_REQUEST),
                                Ok(secret) => {
                                    match storage.set(secret) {
                                        Ok(()) => empty_response(),
                                        Err(err) => ErrorInfo::new(&format!("Storage error: {:?}", err))
                                            .resp(StatusCode::INTERNAL_SERVER_ERROR),
                                    }
                                }
                            }
                        }
                    });
                Either::B(Box::new(resp))
            }
            &Method::DELETE =>
                Either::A(match self.storage.delete(&domain) {
                    Ok(true) => empty_response(),
                    Ok(false) => ErrorInfo::new("Domain not found").resp(StatusCode::NOT_FOUND),
                    Err(err) => ErrorInfo::new(&format!("Storage error: {:?}", err))
                        .resp(StatusCode::INTERNAL_SERVER_ERROR),
                }),
            _ =>
                Either::A(future::ok(json_builder(StatusCode::METHOD_NOT_ALLOWED)
                    .header("Allow", "GET, POST, PUT, DELETE")
                    .body(ErrorInfo::new("Method not allowed").body()).unwrap()))
        }
    }
}
