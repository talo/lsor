use crate::{
    cond::{Eq, Gt, Lt},
    driver::{Driver, PushPrql},
    sort::{Order, Sort},
};

pub fn col(name: &'static str) -> ColumnName {
    ColumnName { name }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ColumnName {
    pub name: &'static str,
}

impl ColumnName {
    pub fn asc(self) -> Sort<Self> {
        Sort {
            order: Order::Asc,
            by: self,
        }
    }

    pub fn desc(self) -> Sort<Self> {
        Sort {
            order: Order::Desc,
            by: self,
        }
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
