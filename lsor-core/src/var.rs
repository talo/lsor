use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::driver::PushPrql;

pub fn zero() -> Literal {
    Literal::I32(0)
}

pub fn one() -> Literal {
    Literal::I32(1)
}

pub fn lit(x: impl Into<Literal>) -> Literal {
    x.into()
}

pub fn null() -> Null {
    Null
}

pub fn empty() -> Empty {
    Empty
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Var {
    Bool(bool),
    I32(i32),
    I64(i64),
    String(String),
    Uuid(Uuid),
    DateTime(DateTime<Utc>),
}

impl PushPrql for Var {
    fn push_to_driver(&self, driver: &mut crate::driver::Driver) {
        match self {
            Self::Bool(x) => driver.push_bind(x),
            Self::I32(x) => driver.push_bind(x),
            Self::I64(x) => driver.push_bind(x),
            Self::String(x) => driver.push_bind(x),
            Self::Uuid(x) => driver.push_bind(x),
            Self::DateTime(x) => driver.push_bind(x),
        };
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Literal {
    Bool(bool),
    I32(i32),
    I64(i64),
    String(String),
    Uuid(Uuid),
}

impl From<bool> for Literal {
    fn from(x: bool) -> Self {
        Self::Bool(x)
    }
}

impl From<i32> for Literal {
    fn from(x: i32) -> Self {
        Self::I32(x)
    }
}

impl From<i64> for Literal {
    fn from(x: i64) -> Self {
        Self::I64(x)
    }
}

impl From<String> for Literal {
    fn from(x: String) -> Self {
        Self::String(x)
    }
}

impl From<Uuid> for Literal {
    fn from(x: Uuid) -> Self {
        Self::Uuid(x)
    }
}

impl PushPrql for Literal {
    fn push_to_driver(&self, driver: &mut crate::driver::Driver) {
        match self {
            Self::Bool(x) => driver.push(x),
            Self::I32(x) => driver.push(x),
            Self::I64(x) => driver.push(x),
            Self::String(x) => driver.push(x),
            Self::Uuid(x) => driver.push(x),
        };
    }
}

pub struct Null;

impl PushPrql for Null {
    fn push_to_driver(&self, driver: &mut crate::driver::Driver) {
        driver.push("null");
    }
}

pub struct Empty;

impl PushPrql for Empty {
    fn push_to_driver(&self, _driver: &mut crate::driver::Driver) {}
}
