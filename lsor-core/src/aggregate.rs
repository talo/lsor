use crate::{column::ColumnName, driver::PushPrql, sort::Sorted};

pub struct Aggregate<Query, Expr> {
    pub query: Query,
    pub aggregations: Vec<(Option<ColumnName>, Expr)>,
}

impl<Query, Expr> Aggregate<Query, Expr> {
    pub fn aggregate(mut self, name: &'static str, expr: Expr) -> Self {
        self.aggregations.push((Some(ColumnName { name }), expr));
        self
    }

    pub fn sort<Sort>(self, sort: Sort) -> Sorted<Self, Sort> {
        Sorted { query: self, sort }
    }
}

impl<Query, Expr> PushPrql for Aggregate<Query, Expr>
where
    Query: PushPrql,
    Expr: PushPrql,
{
    fn push_to_driver(&self, driver: &mut crate::driver::Driver) {
        self.query.push_to_driver(driver);
        driver.push("\naggregate {");
        for (i, (col, expr)) in self.aggregations.iter().enumerate() {
            if i > 0 {
                driver.push(',');
            }
            driver.push(' ');
            if let Some(col) = col {
                col.push_to_driver(driver);
                driver.push(" = ");
            }
            expr.push_to_driver(driver);
        }
        driver.push(" }");
    }
}
