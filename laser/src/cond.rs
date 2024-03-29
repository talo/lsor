use crate::driver::{Driver, PushPrql};

pub fn and<LHS, RHS>(lhs: LHS, rhs: RHS) -> And<LHS, RHS> {
    And { lhs, rhs }
}

pub fn eq<LHS, RHS>(lhs: LHS, rhs: RHS) -> Eq<LHS, RHS> {
    Eq { lhs, rhs }
}

pub fn gt<LHS, RHS>(lhs: LHS, rhs: RHS) -> Gt<LHS, RHS> {
    Gt { lhs, rhs }
}

pub fn lt<LHS, RHS>(lhs: LHS, rhs: RHS) -> Lt<LHS, RHS> {
    Lt { lhs, rhs }
}

pub struct And<LHS, RHS> {
    pub lhs: LHS,
    pub rhs: RHS,
}

impl<LHS, RHS> PushPrql for And<LHS, RHS>
where
    LHS: PushPrql,
    RHS: PushPrql,
{
    fn push_to_driver(&self, driver: &mut Driver) {
        driver.push('(');
        self.lhs.push_to_driver(driver);
        driver.push(") && (");
        self.rhs.push_to_driver(driver);
        driver.push(')');
    }
}

pub struct Eq<LHS, RHS> {
    pub lhs: LHS,
    pub rhs: RHS,
}

impl<LHS, RHS> PushPrql for Eq<LHS, RHS>
where
    LHS: PushPrql,
    RHS: PushPrql,
{
    fn push_to_driver(&self, driver: &mut Driver) {
        driver.push('(');
        self.lhs.push_to_driver(driver);
        driver.push(") == (");
        self.rhs.push_to_driver(driver);
        driver.push(')');
    }
}

pub struct Gt<LHS, RHS> {
    pub lhs: LHS,
    pub rhs: RHS,
}

impl<LHS, RHS> PushPrql for Gt<LHS, RHS>
where
    LHS: PushPrql,
    RHS: PushPrql,
{
    fn push_to_driver(&self, driver: &mut Driver) {
        driver.push('(');
        self.lhs.push_to_driver(driver);
        driver.push(") > (");
        self.rhs.push_to_driver(driver);
        driver.push(')');
    }
}

pub struct Lt<LHS, RHS> {
    pub lhs: LHS,
    pub rhs: RHS,
}

impl<LHS, RHS> PushPrql for Lt<LHS, RHS>
where
    LHS: PushPrql,
    RHS: PushPrql,
{
    fn push_to_driver(&self, driver: &mut Driver) {
        driver.push('(');
        self.lhs.push_to_driver(driver);
        driver.push(") < (");
        self.rhs.push_to_driver(driver);
        driver.push(')');
    }
}
