use crate::{column::ColumnName, driver::PushPrql, empty, Aggregate, Empty};

pub struct Group<Query> {
    pub query: Query,
    pub grouping: Vec<ColumnName>,
}

impl<Query> Group<Query> {
    pub fn aggregate<Expr>(
        self,
        aggregations: impl Into<Vec<(Option<ColumnName>, Expr)>>,
    ) -> GroupPipeline<Query, Aggregate<Empty, Expr>> {
        GroupPipeline {
            query: self.query,
            grouping: self.grouping,
            pipeline: Aggregate {
                query: empty(),
                aggregations: aggregations.into(),
            },
        }
    }
}

pub struct GroupPipeline<Query, Expr> {
    pub query: Query,
    pub grouping: Vec<ColumnName>,
    pub pipeline: Expr,
}

impl<Query, Expr> PushPrql for GroupPipeline<Query, Expr>
where
    Query: PushPrql,
    Expr: PushPrql,
{
    fn push_to_driver(&self, driver: &mut crate::driver::Driver) {
        self.query.push_to_driver(driver);
        driver.push("\ngroup {");
        for (i, col) in self.grouping.iter().enumerate() {
            if i > 0 {
                driver.push(',');
            }
            driver.push(' ');
            col.push_to_driver(driver);
        }
        driver.push(" } (");
        self.pipeline.push_to_driver(driver);
        driver.push(')');
    }
}

#[cfg(test)]
mod test {
    use crate::{column::col, count, from::from, min, null, table::table};

    use super::*;

    #[test]
    fn test_group() {
        let mut driver = crate::driver::Driver::new();
        from(table("jobs"))
            .filter(col("deleted_at").eq(null()))
            .group([col("status"), col("account_id")])
            .aggregate([
                (None, &count() as &dyn PushPrql),
                (
                    Some(col("created_at")),
                    &min(col("created_at")) as &dyn PushPrql,
                ),
            ])
            .push_to_driver(&mut driver);
        assert_eq!(driver.prql(), "from jobs\nfilter (deleted_at) == (null)\ngroup { status, account_id } (\naggregate { count [], created_at = min created_at })");
        assert_eq!(driver.sql(), "SELECT status, account_id, COUNT(*), MIN(created_at) AS created_at FROM jobs WHERE deleted_at IS NULL GROUP BY status, account_id");
    }
}
