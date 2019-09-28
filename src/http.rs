use futures::future::{self, FutureResult};
use hyper::{Body, Response, StatusCode};
use hyper::http::response::Builder;
use serde::Serialize;

use crate::error::Never;

pub type HttpResponse = Response<Body>;

pub fn json<T: ?Sized + Serialize>(value: &T, status: StatusCode) -> FutureResult<HttpResponse, Never> {
    future::ok(Response::builder()
        .header("Content-Type", "application/json")
        .status(status)
        .body(Body::from(serde_json::to_string(value).unwrap()))
        .unwrap())
}

pub fn json_ok<T: ?Sized + Serialize>(value: &T) -> FutureResult<HttpResponse, Never> {
    json(value, StatusCode::OK)
}

pub fn json_builder(status: StatusCode) -> Builder {
    let mut builder = Response::builder();
    builder.header("Content-Type", "application/json").status(status);
    builder
}

pub fn empty_response() -> FutureResult<HttpResponse, Never> {
    future::ok(Response::builder().status(StatusCode::NO_CONTENT)
        .body(Body::from("")).unwrap())
}

