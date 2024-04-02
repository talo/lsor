use crate::{
    driver::{Driver, PushPrql},
    filter::Filtered,
    sort::Sorted,
};

pub struct Taken<Query> {
    pub query: Query,
    pub n: usize,
}

impl<Query> Taken<Query> {
    pub fn filter<Filter>(&self, filter: Filter) -> Filtered<&Self, Filter> {
        Filtered {
            query: self,
            filter,
        }
    }

    pub fn sort<Sort>(&self, sort: Sort) -> Sorted<&Self, Sort> {
        Sorted { query: self, sort }
    }
}

impl<Query> PushPrql for Taken<Query>
where
    Query: PushPrql,
{
    fn push_to_driver(&self, driver: &mut Driver) {
        self.query.push_to_driver(driver);
        driver.push("\ntake ");
        driver.push(self.n);
    }
}

#[cfg(test)]
mod test {
    use crate::{column::col, cond::gt, from::from, table::table};

    use super::*;

    #[test]
    fn test_take() {
        let mut driver = Driver::new();
        {
            from(table("users")).take(10).push_to_driver(&mut driver);
        }
        assert_eq!(driver.sql(), "SELECT * FROM users LIMIT 10");
    }

    #[test]
    fn test_filter_take() {
        let mut driver = Driver::new();
        {
            from(table("users"))
                .filter(gt(col("age"), 18))
                .take(10)
                .push_to_driver(&mut driver);
        }
        assert_eq!(driver.sql(), "SELECT * FROM users WHERE age > $1 LIMIT 10");
    }

    #[test]
    fn test_filter_sort_take() {
        let mut driver = Driver::new();
        {
            from(table("users"))
                .filter(gt(col("age"), 18))
                .sort(col("age").asc())
                .take(10)
                .push_to_driver(&mut driver);
        }
        assert_eq!(
            driver.sql(),
            "SELECT * FROM users WHERE age > $1 ORDER BY age LIMIT 10"
        );
    }

    #[test]
    fn test_sort_take() {
        let mut driver = Driver::new();
        {
            from(table("users"))
                .sort(col("age").asc())
                .take(10)
                .push_to_driver(&mut driver);
        }
        assert_eq!(driver.sql(), "SELECT * FROM users ORDER BY age LIMIT 10");
    }
}
