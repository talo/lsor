use crate::driver::{Driver, PushPrql};

pub fn table(name: &'static str) -> TableName {
    TableName { name }
}

pub trait Table {
    fn table_name() -> TableName;
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TableName {
    pub name: &'static str,
}

impl PushPrql for TableName {
    fn push_to_driver(&self, driver: &mut Driver) {
        driver.push(self.name);
    }
}
