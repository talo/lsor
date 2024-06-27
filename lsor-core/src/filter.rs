use async_graphql::{Enum, OneofObject};
use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::{
    driver::{Driver, PushPrql},
    sort::Sorted,
    take::Taken,
    ColumnName, Derive,
};

pub struct Filtered<Query, Filter> {
    pub query: Query,
    pub filter: Filter,
}

impl<Query, Filter> Filtered<Query, Filter> {
    pub fn sort<Sort>(&self, sort: Sort) -> Sorted<&Self, Sort> {
        Sorted { query: self, sort }
    }

    pub fn take(&self, n: usize) -> Taken<&Self> {
        Taken { query: self, n }
    }

    pub fn derive<Expr>(self, name: &'static str, expr: Expr) -> Derive<Self, Expr> {
        Derive {
            query: self,
            derivations: vec![(ColumnName { name }, expr)],
        }
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

/// This trait is used to define the type `Filter` for some type `Self` that can
/// be used to filter `Self` from the database.
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

impl Filterable for i64 {
    type Filter = I64Filter;
}

impl Filterable for u32 {
    type Filter = I32Filter; // Yes, this is intentional, Postgres doesn't support u32
}

impl Filterable for u64 {
    type Filter = I64Filter; // Yes, this is intentional, Postgres doesn't support u64
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

impl I32Filter {
    pub fn push_to_driver(&self, lhs: &dyn PushPrql, driver: &mut Driver) {
        match self {
            Self::Eq(x) => {
                lhs.push_to_driver(driver);
                driver.push(" == ");
                driver.push_bind(x);
            }
            Self::Ne(x) => {
                lhs.push_to_driver(driver);
                driver.push(" != ");
                driver.push_bind(x);
            }
            Self::Gt(x) => {
                lhs.push_to_driver(driver);
                driver.push(" > ");
                driver.push_bind(x);
            }
            Self::Ge(x) => {
                lhs.push_to_driver(driver);
                driver.push(" >= ");
                driver.push_bind(x);
            }
            Self::Lt(x) => {
                lhs.push_to_driver(driver);
                driver.push(" < ");
                driver.push_bind(x);
            }
            Self::Le(x) => {
                lhs.push_to_driver(driver);
                driver.push(" <= ");
                driver.push_bind(x);
            }
        }
    }

    pub fn push_to_driver_as_json(&self, lhs: &dyn PushPrql, driver: &mut Driver) {
        self.push_to_driver(lhs, driver)
    }
}

#[derive(Clone, Debug, OneofObject)]
#[graphql(rename_fields = "snake_case")]
pub enum I64Filter {
    Eq(i64),
    Ne(i64),
    Gt(i64),
    Ge(i64),
    Lt(i64),
    Le(i64),
}

impl I64Filter {
    pub fn push_to_driver(&self, lhs: &dyn PushPrql, driver: &mut Driver) {
        match self {
            Self::Eq(x) => {
                lhs.push_to_driver(driver);
                driver.push(" == ");
                driver.push_bind(x);
            }
            Self::Ne(x) => {
                lhs.push_to_driver(driver);
                driver.push(" != ");
                driver.push_bind(x);
            }
            Self::Gt(x) => {
                lhs.push_to_driver(driver);
                driver.push(" > ");
                driver.push_bind(x);
            }
            Self::Ge(x) => {
                lhs.push_to_driver(driver);
                driver.push(" >= ");
                driver.push_bind(x);
            }
            Self::Lt(x) => {
                lhs.push_to_driver(driver);
                driver.push(" < ");
                driver.push_bind(x);
            }
            Self::Le(x) => {
                lhs.push_to_driver(driver);
                driver.push(" <= ");
                driver.push_bind(x);
            }
        }
    }

    pub fn push_to_driver_as_json(&self, lhs: &dyn PushPrql, driver: &mut Driver) {
        self.push_to_driver(lhs, driver)
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
    Like(String),
    In(Vec<String>),
}

impl StringFilter {
    pub fn push_to_driver(&self, lhs: &dyn PushPrql, driver: &mut Driver) {
        match self {
            Self::Eq(x) => {
                lhs.push_to_driver(driver);
                driver.push(" == ");
                driver.push_bind(x);
            }
            Self::Ne(x) => {
                lhs.push_to_driver(driver);
                driver.push(" != ");
                driver.push_bind(x);
            }
            Self::Gt(x) => {
                lhs.push_to_driver(driver);
                driver.push(" > ");
                driver.push_bind(x);
            }
            Self::Ge(x) => {
                lhs.push_to_driver(driver);
                driver.push(" >= ");
                driver.push_bind(x);
            }
            Self::Lt(x) => {
                lhs.push_to_driver(driver);
                driver.push(" < ");
                driver.push_bind(x);
            }
            Self::Le(x) => {
                lhs.push_to_driver(driver);
                driver.push(" <= ");
                driver.push_bind(x);
            }
            Self::Like(x) => {
                lhs.push_to_driver(driver);
                driver.push(" s\"");
                driver.push(" LIKE ");
                driver.push_bind(x);
                driver.push('\"');
            }
            Self::In(xs) => {
                lhs.push_to_driver(driver);
                driver.push(" s\"IN (");
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

    pub fn push_to_driver_as_json(&self, lhs: &dyn PushPrql, driver: &mut Driver) {
        match self {
            Self::Like(x) => {
                lhs.push_to_driver(driver);
                driver.push("s\"::text LIKE ");
                driver.push_bind(x);
                driver.push('\"');
            }
            Self::In(xs) => {
                lhs.push_to_driver(driver);
                driver.push(" s\"@> ");
                driver.push_bind(sqlx::types::Json(xs));
                driver.push('\"');
            }
            Self::Eq(x) => {
                lhs.push_to_driver(driver);
                driver.push(" == ");
                driver.push_bind(sqlx::types::Json(x));
            }
            Self::Ne(x) => {
                lhs.push_to_driver(driver);
                driver.push(" != ");
                driver.push_bind(sqlx::types::Json(x));
            }
            Self::Gt(x) => {
                lhs.push_to_driver(driver);
                driver.push(" > ");
                driver.push_bind(sqlx::types::Json(x));
            }
            Self::Ge(x) => {
                lhs.push_to_driver(driver);
                driver.push(" >= ");
                driver.push_bind(sqlx::types::Json(x));
            }
            Self::Lt(x) => {
                lhs.push_to_driver(driver);
                driver.push(" < ");
                driver.push_bind(sqlx::types::Json(x));
            }
            Self::Le(x) => {
                lhs.push_to_driver(driver);
                driver.push(" <= ");
                driver.push_bind(sqlx::types::Json(x));
            }
            _ => self.push_to_driver(lhs, driver),
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

impl UuidFilter {
    pub fn push_to_driver(&self, lhs: &dyn PushPrql, driver: &mut Driver) {
        match self {
            Self::Eq(x) => {
                lhs.push_to_driver(driver);
                driver.push(" == ");
                driver.push_bind(x);
            }
            Self::Ne(x) => {
                lhs.push_to_driver(driver);
                driver.push(" != ");
                driver.push_bind(x);
            }
            Self::In(xs) => {
                lhs.push_to_driver(driver);
                driver.push(" s\"IN (");
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

    pub fn push_to_driver_as_json(&self, lhs: &dyn PushPrql, driver: &mut Driver) {
        match self {
            Self::Eq(x) => {
                lhs.push_to_driver(driver);
                driver.push(" == ");
                driver.push_bind(sqlx::types::Json(x));
            }
            Self::Ne(x) => {
                lhs.push_to_driver(driver);
                driver.push(" != ");
                driver.push_bind(sqlx::types::Json(x));
            }
            Self::In(xs) => {
                lhs.push_to_driver(driver);
                driver.push(" s\"@> ");
                driver.push_bind(sqlx::types::Json(xs));
                driver.push('\"');
            }
        }
    }
}

#[derive(Clone, Debug, OneofObject)]
#[graphql(rename_fields = "snake_case")]
pub enum DateTimeFilter {
    Eq(Option<DateTime<Utc>>),
    Ne(Option<DateTime<Utc>>),
    Gt(DateTime<Utc>),
    Ge(DateTime<Utc>),
    Lt(DateTime<Utc>),
    Le(DateTime<Utc>),
}

impl DateTimeFilter {
    pub fn push_to_driver(&self, lhs: &dyn PushPrql, driver: &mut Driver) {
        match self {
            Self::Eq(x) => {
                lhs.push_to_driver(driver);
                driver.push(" == ");
                match x {
                    Some(x) => driver.push_bind(x),
                    None => driver.push("null"),
                }
            }
            Self::Ne(x) => {
                lhs.push_to_driver(driver);
                driver.push(" != ");
                match x {
                    Some(x) => driver.push_bind(x),
                    None => driver.push("null"),
                }
            }
            Self::Gt(x) => {
                lhs.push_to_driver(driver);
                driver.push(" > ");
                driver.push_bind(x);
            }
            Self::Ge(x) => {
                lhs.push_to_driver(driver);
                driver.push(" >= ");
                driver.push_bind(x);
            }
            Self::Lt(x) => {
                lhs.push_to_driver(driver);
                driver.push(" < ");
                driver.push_bind(x);
            }
            Self::Le(x) => {
                lhs.push_to_driver(driver);
                driver.push(" <= ");
                driver.push_bind(x);
            }
        }
    }

    pub fn push_to_driver_as_json(&self, lhs: &dyn PushPrql, driver: &mut Driver) {
        match self {
            Self::Eq(x) => {
                lhs.push_to_driver(driver);
                driver.push(" == ");
                driver.push_bind(sqlx::types::Json(x));
            }
            Self::Ne(x) => {
                lhs.push_to_driver(driver);
                driver.push(" != ");
                driver.push_bind(sqlx::types::Json(x));
            }
            Self::Gt(x) => {
                lhs.push_to_driver(driver);
                driver.push(" > ");
                driver.push_bind(sqlx::types::Json(x));
            }
            Self::Ge(x) => {
                lhs.push_to_driver(driver);
                driver.push(" >= ");
                driver.push_bind(sqlx::types::Json(x));
            }
            Self::Lt(x) => {
                lhs.push_to_driver(driver);
                driver.push(" < ");
                driver.push_bind(sqlx::types::Json(x));
            }
            Self::Le(x) => {
                lhs.push_to_driver(driver);
                driver.push(" <= ");
                driver.push_bind(sqlx::types::Json(x));
            }
        }
    }
}

#[derive(Clone, Debug, OneofObject)]
#[graphql(rename_fields = "snake_case")]
pub enum TagFilter {
    In(Vec<String>),
}

impl TagFilter {
    pub fn push_to_driver(&self, lhs: &dyn PushPrql, driver: &mut Driver) {
        match self {
            Self::In(xs) => {
                lhs.push_to_driver(driver);
                driver.push(" s\"@> ");
                driver.push_bind(xs);
                driver.push('"');
            }
        }
    }

    pub fn push_to_driver_as_json(&self, lhs: &dyn PushPrql, driver: &mut Driver) {
        match self {
            Self::In(xs) => {
                lhs.push_to_driver(driver);
                driver.push(" s\"@> ");
                driver.push_bind(sqlx::types::Json(xs));
                driver.push('"');
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

impl BoolFilter {
    pub fn push_to_driver(&self, lhs: &dyn PushPrql, driver: &mut Driver) {
        match self {
            Self::True => {
                lhs.push_to_driver(driver);
                driver.push(" == true");
            }
            Self::False => {
                lhs.push_to_driver(driver);
                driver.push(" == false");
            }
        }
    }

    pub fn push_to_driver_as_json(&self, lhs: &dyn PushPrql, driver: &mut Driver) {
        self.push_to_driver(lhs, driver)
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
