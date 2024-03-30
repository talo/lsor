use crate::{
    cond::{Eq, Gt, Lt},
    driver::{Driver, PushPrql},
    sort::Order,
};

pub fn col(name: &'static str) -> ColumnName {
    ColumnName { name }
}

pub struct ColumnName {
    pub name: &'static str,
}

impl ColumnName {
    pub fn asc(self) -> Order<Self> {
        Order::Asc(self)
    }

    pub fn desc(self) -> Order<Self> {
        Order::Desc(self)
    }

    pub fn eq<RHS>(&self, rhs: RHS) -> Eq<&Self, RHS> {
        Eq { lhs: self, rhs }
    }

    pub fn gt<RHS>(&self, rhs: RHS) -> Gt<&Self, RHS> {
        Gt { lhs: self, rhs }
    }

    pub fn lt<RHS>(&self, rhs: RHS) -> Lt<&Self, RHS> {
        Lt { lhs: self, rhs }
    }
}

impl PushPrql for ColumnName {
    fn push_to_driver(&self, driver: &mut Driver) {
        driver.push(self.name);
    }
}
