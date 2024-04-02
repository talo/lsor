use crate::{
    cond::{Eq, Gt, Lt},
    driver::{Driver, PushPrql},
    sort::{Order, Sort},
};

pub fn col(name: &'static str) -> ColumnName {
    ColumnName { name }
}

pub fn json<Col>(col: Col) -> Json<Col> {
    Json { col }
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Json<Col> {
    pub col: Col,
}

impl<Col> Json<Col> {
    pub fn at(self, n: usize) -> JsonAccessor<Col> {
        JsonAccessor {
            col: self.col,
            op: JsonOp::At(n),
        }
    }

    pub fn get(self, k: &'static str) -> JsonAccessor<Col> {
        JsonAccessor {
            col: self.col,
            op: JsonOp::Get(k),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum JsonOp {
    At(usize),         // json -> integer -> json
    Get(&'static str), // json -> text -> json
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct JsonAccessor<Col> {
    pub col: Col,
    pub op: JsonOp,
}

impl<Col> JsonAccessor<Col> {
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

impl<Col> PushPrql for JsonAccessor<Col>
where
    Col: PushPrql,
{
    fn push_to_driver(&self, driver: &mut Driver) {
        driver.push("s\"");
        self.col.push_to_driver(driver);
        driver.push("->");
        match self.op {
            JsonOp::At(n) => {
                driver.push(n);
            }
            JsonOp::Get(k) => {
                driver.push("'");
                driver.push(k);
                driver.push("'");
            }
        }
        driver.push('"');
    }
}
