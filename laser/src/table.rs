use std::fmt::Display;

use crate::driver::{Driver, PushPrql};

pub fn table(name: impl Display) -> TableName {
    TableName {
        name: name.to_string(),
    }
}

pub trait Table {
    fn table() -> TableName;
}

pub struct TableName {
    pub name: String,
}

impl PushPrql for TableName {
    fn push_to_driver(&self, driver: &mut Driver) {
        driver.push(&self.name);
    }
}
