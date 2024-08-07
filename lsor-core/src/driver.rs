use std::fmt::Display;

use chrono::{DateTime, Utc};
use sqlx::{postgres::PgArguments, Database, Encode, Executor, Postgres, Type};
use uuid::Uuid;

use crate::Cache;

pub struct Driver {
    prql: String,
    arguments: PgArguments,
    cache: Option<Box<dyn Cache>>,
}

impl Driver {
    pub fn new() -> Self {
        Driver {
            prql: String::new(),
            arguments: PgArguments::default(),
            cache: None,
        }
    }

    pub fn with_cache(cache: Box<dyn Cache>) -> Self {
        Driver {
            prql: String::new(),
            arguments: PgArguments::default(),
            cache: Some(cache.into()),
        }
    }

    pub fn prql(&self) -> &str {
        &self.prql
    }

    pub fn sql(&self) -> String {
        use prqlc::{sql::Dialect, Options, Target};

        let opts = &Options {
            format: false,
            signature_comment: false,
            target: Target::Sql(Some(Dialect::Postgres)),
            ..Default::default()
        };

        let cached_sql = self.fetch_from_cache(&self.prql);
        if let Some(cached_sql) = cached_sql {
            tracing::debug!("returning cached sql:\n{}", &cached_sql);
            return cached_sql.clone();
        }

        match prqlc::compile(&self.prql, opts) {
            Ok(sql) => {
                tracing::debug!("compiling prql:\n{}\ninto sql:\n{}", &self.prql, &sql);
                self.add_to_cache(self.prql.clone(), sql.clone());
                sql
            }
            Err(e) => {
                tracing::error!("bad prql:\n{}", &self.prql);
                panic!("must compile prql: {}", e)
            }
        }
    }

    pub fn is_empty(&self) -> bool {
        self.prql.is_empty()
    }

    pub fn push(&mut self, prql: impl Display) {
        use std::fmt::Write as _;

        write!(&mut self.prql, "{}", prql).expect("must write pqrl");
    }

    pub fn push_bind<T>(&mut self, value: T)
    where
        for<'q> T: Encode<'q, Postgres> + Send + Type<Postgres>,
    {
        use sqlx::Arguments as _;

        self.arguments.add(value);
        self.arguments
            .format_placeholder(&mut self.prql)
            .expect("must format placeholder");
    }

    pub async fn execute_without_compilation<'c>(
        self,
        executor: impl Executor<'c, Database = Postgres>,
    ) -> sqlx::Result<<Postgres as Database>::QueryResult> {
        use sqlx::QueryBuilder;

        QueryBuilder::with_arguments(self.prql, self.arguments)
            .build()
            .execute(executor)
            .await
    }

    pub async fn fetch_all(
        self,
        executor: impl Executor<'_, Database = Postgres>,
    ) -> sqlx::Result<Vec<<Postgres as Database>::Row>> {
        use sqlx::QueryBuilder;

        QueryBuilder::with_arguments(self.sql(), self.arguments)
            .build()
            .fetch_all(executor)
            .await
    }

    pub async fn fetch_one(
        self,
        executor: impl Executor<'_, Database = Postgres>,
    ) -> sqlx::Result<<Postgres as Database>::Row> {
        use sqlx::QueryBuilder;

        QueryBuilder::with_arguments(self.sql(), self.arguments)
            .build()
            .fetch_one(executor)
            .await
    }

    pub async fn fetch_optional(
        self,
        executor: impl Executor<'_, Database = Postgres>,
    ) -> sqlx::Result<Option<<Postgres as Database>::Row>> {
        use sqlx::QueryBuilder;

        QueryBuilder::with_arguments(self.sql(), self.arguments)
            .build()
            .fetch_optional(executor)
            .await
    }

    fn add_to_cache(&self, key: String, value: String) {
        if let Some(cache) = &self.cache {
            cache.insert(key, value);
        }
    }

    fn fetch_from_cache(&self, key: &String) -> Option<String> {
        self.cache.as_ref().and_then(|cache| cache.get(key))
    }
}

impl Default for Driver {
    fn default() -> Self {
        Self::new()
    }
}

pub trait PushPrql {
    fn push_to_driver(&self, driver: &mut Driver);
}

impl PushPrql for String {
    fn push_to_driver(&self, driver: &mut Driver) {
        driver.push_bind(self);
    }
}

impl PushPrql for &str {
    fn push_to_driver(&self, driver: &mut Driver) {
        driver.push_bind(self);
    }
}

impl PushPrql for i32 {
    fn push_to_driver(&self, driver: &mut Driver) {
        driver.push_bind(self);
    }
}

impl PushPrql for i64 {
    fn push_to_driver(&self, driver: &mut Driver) {
        driver.push_bind(self);
    }
}

impl PushPrql for u32 {
    fn push_to_driver(&self, driver: &mut Driver) {
        driver.push_bind(*self as i32);
    }
}

impl PushPrql for u64 {
    fn push_to_driver(&self, driver: &mut Driver) {
        driver.push_bind(*self as i64);
    }
}

impl PushPrql for f32 {
    fn push_to_driver(&self, driver: &mut Driver) {
        driver.push_bind(*self);
    }
}

impl PushPrql for f64 {
    fn push_to_driver(&self, driver: &mut Driver) {
        driver.push_bind(*self);
    }
}

impl PushPrql for bool {
    fn push_to_driver(&self, driver: &mut Driver) {
        driver.push_bind(self);
    }
}

impl PushPrql for Uuid {
    fn push_to_driver(&self, driver: &mut Driver) {
        driver.push_bind(self);
    }
}

impl PushPrql for DateTime<Utc> {
    fn push_to_driver(&self, driver: &mut Driver) {
        driver.push_bind(self);
    }
}

impl<T> PushPrql for Option<T>
where
    for<'q> T: 'q + Encode<'q, Postgres> + Sync + Type<Postgres>,
{
    fn push_to_driver(&self, driver: &mut Driver) {
        driver.push_bind(self);
    }
}

impl<T> PushPrql for Vec<T>
where
    T: PushPrql,
{
    fn push_to_driver(&self, driver: &mut Driver) {
        for (i, value) in self.iter().enumerate() {
            if i > 0 {
                driver.push(", ");
            }
            value.push_to_driver(driver);
        }
    }
}

impl<T> PushPrql for &T
where
    T: PushPrql,
{
    fn push_to_driver(&self, driver: &mut Driver) {
        (*self).push_to_driver(driver);
    }
}

impl PushPrql for &dyn PushPrql {
    fn push_to_driver(&self, driver: &mut Driver) {
        (*self).push_to_driver(driver)
    }
}

pub fn sql(sql: &'static str) -> SQL {
    SQL { sql }
}

pub struct SQL {
    pub sql: &'static str,
}

impl PushPrql for SQL {
    fn push_to_driver(&self, driver: &mut crate::driver::Driver) {
        driver.push("s\"");
        driver.push(self.sql);
        driver.push('\"');
    }
}
