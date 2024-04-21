use crate::{column::ColumnName, driver::PushPrql, sort::Sorted};

pub struct Derive<Query, Expr> {
    pub query: Query,
    pub derivations: Vec<(ColumnName, Expr)>,
}

impl<Query, Expr> Derive<Query, Expr> {
    pub fn derive(mut self, name: &'static str, expr: Expr) -> Self {
        self.derivations.push((ColumnName { name }, expr));
        self
    }

    pub fn sort<Sort>(self, sort: Sort) -> Sorted<Self, Sort> {
        Sorted { query: self, sort }
    }
}

impl<Query, Expr> PushPrql for Derive<Query, Expr>
where
    Query: PushPrql,
    Expr: PushPrql,
{
    fn push_to_driver(&self, driver: &mut crate::driver::Driver) {
        self.query.push_to_driver(driver);
        driver.push("\nderive {");
        for (i, (col, expr)) in self.derivations.iter().enumerate() {
            if i > 0 {
                driver.push(',');
            }
            driver.push(' ');
            col.push_to_driver(driver);
            driver.push(" = ");
            expr.push_to_driver(driver);
        }
        driver.push(" }");
    }
}
