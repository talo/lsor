use crate::{
    driver::{Driver, PushPrql},
    take::Taken,
};

pub trait Ordered {
    type By;

    fn order(&self) -> &Order<Self::By>;
}

pub enum Order<By> {
    Asc(By),
    Desc(By),
}

impl<By> Order<By> {
    pub fn by(&self) -> &By {
        match self {
            Order::Asc(by) => by,
            Order::Desc(by) => by,
        }
    }

    pub fn flip(&self) -> Order<&By> {
        match self {
            Order::Asc(by) => Order::Desc(by),
            Order::Desc(by) => Order::Asc(by),
        }
    }

    pub fn as_ref(&self) -> Order<&By> {
        match self {
            Order::Asc(by) => Order::Asc(by),
            Order::Desc(by) => Order::Desc(by),
        }
    }

    pub fn is_asc(&self) -> bool {
        matches!(self, Order::Asc(_))
    }

    pub fn is_desc(&self) -> bool {
        matches!(self, Order::Desc(_))
    }
}

impl<By> PushPrql for Order<By>
where
    By: PushPrql,
{
    fn push_to_driver(&self, driver: &mut Driver) {
        match self {
            Order::Asc(by) => by.push_to_driver(driver),
            Order::Desc(by) => {
                driver.push("-");
                by.push_to_driver(driver);
            }
        }
    }
}

pub struct Sorted<Query, By> {
    pub query: Query,
    pub order: Order<By>,
}

impl<Query, By> Sorted<Query, By> {
    pub fn take(&self, n: usize) -> Taken<&Self> {
        Taken { query: self, n }
    }
}

impl<Query, By> Ordered for Sorted<Query, By> {
    type By = By;

    fn order(&self) -> &Order<Self::By> {
        &self.order
    }
}

impl<Query, Sort> PushPrql for Sorted<Query, Sort>
where
    Query: PushPrql,
    Sort: PushPrql,
{
    fn push_to_driver(&self, driver: &mut Driver) {
        self.query.push_to_driver(driver);
        driver.push("\nsort {");
        self.order.push_to_driver(driver);
        driver.push("}");
    }
}

#[cfg(test)]
mod test {
    use crate::{column::col, cond::gt, from::from, table::table};

    use super::*;

    #[test]
    fn test_sort() {
        let mut driver = Driver::new();
        {
            from(table("users"))
                .sort(col("age").asc())
                .push_to_driver(&mut driver);
        }
        assert_eq!(driver.sql(), "SELECT * FROM users ORDER BY age");
    }

    #[test]
    fn test_sort_desc() {
        let mut driver = Driver::new();
        {
            from(table("users"))
                .sort(col("age").desc())
                .push_to_driver(&mut driver);
        }
        dbg!(driver.prql());
        assert_eq!(driver.sql(), "SELECT * FROM users ORDER BY age DESC");
    }

    #[test]
    fn test_filter_sort() {
        let mut driver = Driver::new();
        {
            from(table("users"))
                .filter(gt(col("age"), 18))
                .sort(col("age").asc())
                .push_to_driver(&mut driver);
        }
        assert_eq!(
            driver.sql(),
            "SELECT * FROM users WHERE age > $1 ORDER BY age"
        );
    }

    #[test]
    fn test_take_sort() {
        let mut driver = Driver::new();
        {
            from(table("users"))
                .take(10)
                .sort(col("age").asc())
                .push_to_driver(&mut driver);
        }
        assert_eq!(
            driver.sql(),
            "SELECT * FROM users WHERE age > $1 ORDER BY age"
        );
    }
}
