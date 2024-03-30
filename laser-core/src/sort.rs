use crate::{
    driver::{Driver, PushPrql},
    take::Taken,
};

pub trait SortedBy {
    type By;

    fn sorted_by(&self) -> &Sort<Self::By>;
}

impl<T> SortedBy for &T
where
    T: SortedBy,
{
    type By = T::By;

    fn sorted_by(&self) -> &Sort<Self::By> {
        (*self).sorted_by()
    }
}

pub enum Sort<By> {
    Asc(By),
    Desc(By),
}

impl<By> Sort<By> {
    pub fn by(&self) -> &By {
        match self {
            Sort::Asc(by) => by,
            Sort::Desc(by) => by,
        }
    }

    pub fn flip(&self) -> Sort<&By> {
        match self {
            Sort::Asc(by) => Sort::Desc(by),
            Sort::Desc(by) => Sort::Asc(by),
        }
    }

    pub fn as_ref(&self) -> Sort<&By> {
        match self {
            Sort::Asc(by) => Sort::Asc(by),
            Sort::Desc(by) => Sort::Desc(by),
        }
    }

    pub fn is_asc(&self) -> bool {
        matches!(self, Sort::Asc(_))
    }

    pub fn is_desc(&self) -> bool {
        matches!(self, Sort::Desc(_))
    }
}

impl<By> PushPrql for Sort<By>
where
    By: PushPrql,
{
    fn push_to_driver(&self, driver: &mut Driver) {
        match self {
            Sort::Asc(by) => by.push_to_driver(driver),
            Sort::Desc(by) => {
                driver.push("-");
                by.push_to_driver(driver);
            }
        }
    }
}

pub struct Sorted<Query, By> {
    pub query: Query,
    pub sort: Sort<By>,
}

impl<Query, By> Sorted<Query, By> {
    pub fn take(&self, n: usize) -> Taken<&Self> {
        Taken { query: self, n }
    }
}

impl<Query, By> SortedBy for Sorted<Query, By> {
    type By = By;

    fn sorted_by(&self) -> &Sort<Self::By> {
        &self.sort
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
        self.sort.push_to_driver(driver);
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
