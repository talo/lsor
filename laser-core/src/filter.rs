use async_graphql::{Enum, OneofObject};
use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::{
    driver::{Driver, PushPrql},
    sort::{Sort, Sorted},
    take::Taken,
};

pub struct Filtered<Query, Filter> {
    pub query: Query,
    pub filter: Filter,
}

impl<Query, Filter> Filtered<Query, Filter> {
    pub fn sort<By>(&self, sort: Sort<By>) -> Sorted<&Self, By> {
        Sorted { query: self, sort }
    }

    pub fn take(&self, n: usize) -> Taken<&Self> {
        Taken { query: self, n }
    }
}

impl<Query, Filter> PushPrql for Filtered<Query, Filter>
where
    Query: PushPrql,
    Filter: PushPrql,
{
    fn push_to_driver(&self, driver: &mut Driver) {
        self.query.push_to_driver(driver);
        driver.push("\nfilter ");
        self.filter.push_to_driver(driver);
    }
}

pub trait Filterable {
    type Filter;
}

impl<T> Filterable for Option<T>
where
    T: Filterable,
{
    type Filter = <T as Filterable>::Filter;
}

impl Filterable for i32 {
    type Filter = I32Filter;
}

impl Filterable for String {
    type Filter = StringFilter;
}

impl Filterable for Uuid {
    type Filter = UuidFilter;
}

impl Filterable for DateTime<Utc> {
    type Filter = DateTimeFilter;
}

impl Filterable for Vec<String> {
    type Filter = TagFilter;
}

impl Filterable for bool {
    type Filter = BoolFilter;
}

#[derive(Clone, Debug, OneofObject)]
#[graphql(rename_fields = "snake_case")]
pub enum I32Filter {
    Eq(i32),
    Ne(i32),
    Gt(i32),
    Ge(i32),
    Lt(i32),
    Le(i32),
}

impl PushPrql for I32Filter {
    fn push_to_driver(&self, driver: &mut Driver) {
        match self {
            Self::Eq(x) => {
                driver.push(" == ");
                driver.push_bind(x);
            }
            Self::Ne(x) => {
                driver.push(" != ");
                driver.push_bind(x);
            }
            Self::Gt(x) => {
                driver.push(" > ");
                driver.push_bind(x);
            }
            Self::Ge(x) => {
                driver.push(" >= ");
                driver.push_bind(x);
            }
            Self::Lt(x) => {
                driver.push(" < ");
                driver.push_bind(x);
            }
            Self::Le(x) => {
                driver.push(" <= ");
                driver.push_bind(x);
            }
        }
    }
}

#[derive(Clone, Debug, OneofObject)]
#[graphql(rename_fields = "snake_case")]
pub enum StringFilter {
    Eq(String),
    Ne(String),
    Gt(String),
    Ge(String),
    Lt(String),
    Le(String),
}

impl PushPrql for StringFilter {
    fn push_to_driver(&self, driver: &mut Driver) {
        match self {
            Self::Eq(x) => {
                driver.push(" == ");
                driver.push_bind(x);
            }
            Self::Ne(x) => {
                driver.push(" != ");
                driver.push_bind(x);
            }
            Self::Gt(x) => {
                driver.push(" > ");
                driver.push_bind(x);
            }
            Self::Ge(x) => {
                driver.push(" >= ");
                driver.push_bind(x);
            }
            Self::Lt(x) => {
                driver.push(" < ");
                driver.push_bind(x);
            }
            Self::Le(x) => {
                driver.push(" <= ");
                driver.push_bind(x);
            }
        }
    }
}

#[derive(Clone, Debug, OneofObject)]
#[graphql(rename_fields = "snake_case")]
pub enum UuidFilter {
    Eq(Uuid),
    Ne(Uuid),
    In(Vec<Uuid>),
}

impl PushPrql for UuidFilter {
    fn push_to_driver(&self, driver: &mut Driver) {
        match self {
            Self::Eq(x) => {
                driver.push(" == ");
                driver.push_bind(x);
            }
            Self::Ne(x) => {
                driver.push(" != ");
                driver.push_bind(x);
            }
            Self::In(xs) => {
                driver.push("s\"");
                driver.push(" IN (");
                for (i, x) in xs.iter().enumerate() {
                    if i > 0 {
                        driver.push(", ");
                    }
                    driver.push_bind(x);
                }
                driver.push(")\"");
            }
        }
    }
}

#[derive(Clone, Debug, OneofObject)]
#[graphql(rename_fields = "snake_case")]
pub enum DateTimeFilter {
    Eq(DateTime<Utc>),
    Ne(DateTime<Utc>),
    Gt(DateTime<Utc>),
    Ge(DateTime<Utc>),
    Lt(DateTime<Utc>),
    Le(DateTime<Utc>),
}

impl PushPrql for DateTimeFilter {
    fn push_to_driver(&self, driver: &mut Driver) {
        match self {
            Self::Eq(x) => {
                driver.push(" == ");
                driver.push_bind(x);
            }
            Self::Ne(x) => {
                driver.push(" != ");
                driver.push_bind(x);
            }
            Self::Gt(x) => {
                driver.push(" > ");
                driver.push_bind(x);
            }
            Self::Ge(x) => {
                driver.push(" >= ");
                driver.push_bind(x);
            }
            Self::Lt(x) => {
                driver.push(" < ");
                driver.push_bind(x);
            }
            Self::Le(x) => {
                driver.push(" <= ");
                driver.push_bind(x);
            }
        }
    }
}

#[derive(Clone, Debug, OneofObject)]
#[graphql(rename_fields = "snake_case")]
pub enum TagFilter {
    In(Vec<String>),
}

impl PushPrql for TagFilter {
    fn push_to_driver(&self, driver: &mut Driver) {
        match self {
            Self::In(xs) => {
                driver.push("s\" @> ");
                driver.push_bind(xs);
                driver.push('\"');
            }
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Enum)]
#[graphql(rename_items = "snake_case")]
pub enum BoolFilter {
    True,
    False,
}

impl PushPrql for BoolFilter {
    fn push_to_driver(&self, driver: &mut Driver) {
        match self {
            Self::True => {
                driver.push(" == true");
            }
            Self::False => {
                driver.push(" == false");
            }
        }
    }
}

#[cfg(test)]
mod test {
    use crate::{column::col, cond::gt, from::from, table::table};

    use super::*;

    #[test]
    fn test_filter() {
        let mut driver = Driver::new();
        {
            from(table("users"))
                .filter(gt(col("age"), 18))
                .push_to_driver(&mut driver);
        }
        assert_eq!(driver.sql(), "SELECT * FROM users WHERE age > $1");
    }

    #[test]
    fn test_sort_filter() {
        let mut driver = Driver::new();
        {
            from(table("users"))
                .sort(col("age").asc())
                .push_to_driver(&mut driver);
        }
        assert_eq!(driver.sql(), "SELECT * FROM users ORDER BY age");
    }

    #[test]
    fn test_take_filter() {
        let mut driver = Driver::new();
        {
            from(table("users"))
                .take(10)
                .filter(gt(col("age"), 18))
                .push_to_driver(&mut driver);
        }
        assert_eq!(
            driver.sql(),
            "WITH table_0 AS (SELECT * FROM users LIMIT 10) SELECT * FROM table_0 WHERE age > $1"
        );
    }
}
