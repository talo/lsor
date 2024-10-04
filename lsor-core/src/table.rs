use crate::{
    driver::{Driver, PushPrql},
    Order, Sort,
};

pub fn dot<LHS, RHS>(lhs: LHS, rhs: RHS) -> Dot<LHS, RHS> {
    Dot { lhs, rhs }
}

pub fn table(name: &'static str) -> TableName {
    TableName { name }
}

pub trait Table {
    fn table_name() -> TableName;
}

impl<T> Table for &T
where
    T: Table,
{
    fn table_name() -> TableName {
        T::table_name()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TableName {
    pub name: &'static str,
}

impl TableName {
    pub fn dot<RHS>(&self, rhs: RHS) -> Dot<&Self, RHS> {
        Dot { lhs: self, rhs }
    }
}

impl PushPrql for TableName {
    fn push_to_driver(&self, driver: &mut Driver) {
        driver.push(self.name);
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Dot<LHS, RHS> {
    pub lhs: LHS,
    pub rhs: RHS,
}

impl<LHS, RHS> Dot<LHS, RHS> {
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
}

impl<LHS, RHS> PushPrql for Dot<LHS, RHS>
where
    LHS: PushPrql,
    RHS: PushPrql,
{
    fn push_to_driver(&self, driver: &mut Driver) {
        self.lhs.push_to_driver(driver);
        driver.push('.');
        self.rhs.push_to_driver(driver);
    }
}
