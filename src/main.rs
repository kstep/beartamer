#[macro_use]
extern crate serde_derive;

use std::env::args;
use std::error::Error;
use std::fmt;
use std::net::SocketAddr;
use std::str::FromStr;

use futures::{future, Future, Stream};
use futures::future::{FutureResult, Either};
use hyper::{Method, Request, Response, rt, Server, StatusCode};
use hyper::body::Body;
use hyper::http::response::Builder;
use hyper::service::{Service, service_fn_ok, make_service_fn};
use log::{error, warn};
use serde::Serialize;

pub enum Never {}
impl fmt::Display for Never {
    fn fmt(&self, _f: &mut fmt::Formatter<'_>) -> fmt::Result { match *self {} }
}
impl fmt::Debug for Never {
    fn fmt(&self, _f: &mut fmt::Formatter<'_>) -> fmt::Result { match *self {} }
}
impl Error for Never {}

#[derive(Serialize, Deserialize, Debug)]
struct Secret {
    r#type: SecretType,
    domain: String,
    username: String,
    password: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
enum SecretType {
    Password
}

#[derive(Serialize)]
struct ErrorInfo<'a> {
    message: &'a str,
}

impl<'a> ErrorInfo<'a> {
    fn new(message: &'a str) -> Self {
        ErrorInfo { message }
    }
    fn json(&self) -> String {
        serde_json::to_string(self).unwrap()
    }
    fn body(&self) -> Body {
        Body::from(self.json())
    }
    fn resp(&self, status: StatusCode) -> FutureResult<HttpResponse, Never> {
        json(self, status)
    }
    fn resp_ok(&self) -> FutureResult<HttpResponse, Never> {
        json_ok(self)
    }
}

pub type HttpResponse = Response<Body>;

static DEFAULT_BIND: &str = "127.0.0.1:9000";

fn main() {
    let address = {
        let addr = args().next().unwrap_or_else(|| {
            warn!("No binding given, using default {}", DEFAULT_BIND);
            String::from(DEFAULT_BIND)
        });
        SocketAddr::from_str(&addr).or_else(|err| {
            warn!("Invalid binding given ({}), using default {}", err, DEFAULT_BIND);
            SocketAddr::from_str(DEFAULT_BIND)
        }).unwrap()
    };

    let server = Server::bind(&address)
        .serve(make_service_fn(|_| future::ok::<_, Never>(SecretService)))
        .map_err(|e| error!("Error: {:?}", e));

    rt::run(rt::lazy(move || {
        rt::spawn(server);
        Ok(())
    }));
}

fn json<T: ?Sized + Serialize>(value: &T, status: StatusCode) -> FutureResult<HttpResponse, Never> {
    future::ok(Response::builder()
        .header("Content-Type", "application/json")
        .status(status)
        .body(Body::from(serde_json::to_string(value).unwrap()))
        .unwrap())
}

fn json_ok<T: ?Sized + Serialize>(value: &T) -> FutureResult<HttpResponse, Never> {
    json(value, StatusCode::OK)
}

fn json_builder(status: StatusCode) -> Builder {
    let mut builder = Response::builder();
    builder.header("Content-Type", "application/json").status(status);
    builder
}

pub struct SecretService;

impl Service for SecretService {
    type ReqBody = Body;
    type ResBody = Body;
    type Error = Never;
    type Future = Either<FutureResult<Response<Body>, Never>, Box<dyn Future<Item=Response<Body>, Error=Never> + Send>>;

    fn call(&mut self, req: Request<Self::ReqBody>) -> Self::Future {
        let mut path = req.uri().path().trim_start_matches("/").split("/");
        let domain = match path.next() {
            None => return Either::A(ErrorInfo::new("Domain name missing").resp(StatusCode::BAD_REQUEST)),
            Some(domain) => domain.to_string(),
        };
        let method = req.method();
        match method {
            &Method::GET =>
                Either::A(json_ok(&Secret {
                    r#type: SecretType::Password,
                    domain: domain,
                    username: "".to_string(),
                    password: "".to_string(),
                })),
            &Method::PUT | &Method::POST => {
                let body = req.into_body()
                    .concat2()
                    .map(|c| {
                        match String::from_utf8(c.to_vec()) {
                            Err(err) => ErrorInfo::new(&format!("invalid data: {}", err)).json(),
                            Ok(s) => match serde_json::from_str::<Secret>(&s) {
                                Err(err) => ErrorInfo::new(&format!("invalid json: {}", err)).json(),
                                Ok(s) => serde_json::to_string(&s).unwrap(),
                            }
                        }
                    }).into_stream();
                Either::A(future::ok(Response::new(Body::wrap_stream(body))))
            },
            &Method::DELETE =>
                Either::A(ErrorInfo::new("Domain not found").resp(StatusCode::NOT_FOUND)),
            // json_ok(&ErrorInfo { message: "Domain deleted" }),
            _ =>
                Either::A(future::ok(json_builder(StatusCode::METHOD_NOT_ALLOWED)
                    .header("Allow", "GET, POST, PUT, DELETE")
                    .body(ErrorInfo::new("Method not allowed").body()).unwrap()))
        }
    }
}
