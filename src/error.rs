use std::error::Error;
use std::fmt;

use futures::future::FutureResult;
use hyper::{Body, StatusCode};

use crate::http::{HttpResponse, json, json_ok};

pub enum Never {}

impl fmt::Display for Never {
    fn fmt(&self, _f: &mut fmt::Formatter<'_>) -> fmt::Result { match *self {} }
}

impl fmt::Debug for Never {
    fn fmt(&self, _f: &mut fmt::Formatter<'_>) -> fmt::Result { match *self {} }
}

impl Error for Never {}

#[derive(Serialize)]
pub struct ErrorInfo<'a> {
    message: &'a str,
}

impl<'a> ErrorInfo<'a> {
    pub fn new(message: &'a str) -> Self {
        ErrorInfo { message }
    }
    pub fn json(&self) -> String {
        serde_json::to_string(self).unwrap()
    }
    pub fn body(&self) -> Body {
        Body::from(self.json())
    }
    pub fn resp(&self, status: StatusCode) -> FutureResult<HttpResponse, Never> {
        json(self, status)
    }
    pub fn resp_ok(&self) -> FutureResult<HttpResponse, Never> {
        json_ok(self)
    }
}

