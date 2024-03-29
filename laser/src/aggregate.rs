use crate::{
    column::ColumnName,
    driver::PushPrql,
    sort::{Order, Sorted},
};

pub struct Aggregate<Query, Expr> {
    pub query: Query,
    pub aggregations: Vec<(ColumnName, Expr)>,
}

impl<Query, Expr> Aggregate<Query, Expr> {
    pub fn aggregate(mut self, name: &'static str, expr: Expr) -> Self {
        self.aggregations.push((ColumnName { name }, expr));
        self
    }

    pub fn sort<By>(self, order: Order<By>) -> Sorted<Self, By> {
        Sorted { query: self, order }
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
            col.push_to_driver(driver);
            driver.push(" = ");
            expr.push_to_driver(driver);
        }
        driver.push(" }");
    }
}
