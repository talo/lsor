use std::fmt::Display;

use sqlx::{postgres::PgArguments, Database, Encode, Executor, Postgres, Type};

pub struct Driver {
    prql: String,
    arguments: PgArguments,
}

impl Driver {
    pub fn new() -> Self {
        Driver {
            prql: String::new(),
            arguments: PgArguments::default(),
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

        prqlc::compile(&self.prql, opts).expect("must compile prql")
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

    pub async fn execute<'c>(
        self,
        executor: impl Executor<'c, Database = Postgres>,
    ) -> sqlx::Result<<Postgres as Database>::QueryResult> {
        use sqlx::QueryBuilder;

        QueryBuilder::with_arguments(self.sql(), self.arguments)
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
}

impl Default for Driver {
    fn default() -> Self {
        Self::new()
    }
}

pub trait PushPrql {
    fn push_to_driver(&self, driver: &mut Driver);
}

impl<T> PushPrql for &T
where
    T: PushPrql,
{
    fn push_to_driver(&self, driver: &mut Driver) {
        (*self).push_to_driver(driver)
    }
}

impl PushPrql for &dyn PushPrql {
    fn push_to_driver(&self, driver: &mut Driver) {
        (*self).push_to_driver(driver)
    }
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

impl PushPrql for bool {
    fn push_to_driver(&self, driver: &mut Driver) {
        driver.push_bind(self);
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
