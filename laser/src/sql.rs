use chrono::{DateTime, Utc};
use sqlx::{Postgres, QueryBuilder};
use uuid::Uuid;

pub trait IntoSql {
    fn into_sql(self, qb: &mut QueryBuilder<'_, Postgres>);
}

// impl<T> IntoSql for &T
// where
//     T: Copy + IntoSql,
// {
//     fn into_sql(self, qb: &mut QueryBuilder<'_, Postgres>) {
//         (*self).into_sql(qb);
//     }
// }

impl<T> IntoSql for Vec<T>
where
    for<'args> T: 'args
        + ::sqlx::Encode<'args, ::sqlx::Postgres>
        + ::sqlx::postgres::PgHasArrayType
        + Send
        + ::sqlx::Type<::sqlx::Postgres>,
{
    fn into_sql(self, qb: &mut QueryBuilder<'_, Postgres>) {
        qb.push_bind(self);
    }
}

impl<T> IntoSql for Option<T>
where
    T: IntoSql,
{
    fn into_sql(self, qb: &mut QueryBuilder<'_, Postgres>) {
        match self {
            Some(value) => {
                value.into_sql(qb);
            }
            None => {
                qb.push("NULL");
            }
        }
    }
}

impl IntoSql for bool {
    fn into_sql(self, qb: &mut QueryBuilder<'_, Postgres>) {
        qb.push_bind(self);
    }
}

impl IntoSql for i32 {
    fn into_sql(self, qb: &mut QueryBuilder<'_, Postgres>) {
        qb.push_bind(self);
    }
}

impl IntoSql for i64 {
    fn into_sql(self, qb: &mut QueryBuilder<'_, Postgres>) {
        qb.push_bind(self);
    }
}

impl IntoSql for String {
    fn into_sql(self, qb: &mut QueryBuilder<'_, Postgres>) {
        qb.push_bind(self);
    }
}

impl IntoSql for &'static str {
    fn into_sql(self, qb: &mut QueryBuilder<'_, Postgres>) {
        qb.push_bind(self);
    }
}

impl IntoSql for Uuid {
    fn into_sql(self, qb: &mut QueryBuilder<'_, Postgres>) {
        qb.push_bind(self);
    }
}

impl IntoSql for DateTime<Utc> {
    fn into_sql(self, qb: &mut QueryBuilder<'_, Postgres>) {
        qb.push_bind(self);
    }
}

// impl<T> IntoSql for T
// where
//     for<'args> T:
//         'args + ::sqlx::Encode<'args, ::sqlx::Postgres> + Send + ::sqlx::Type<::sqlx::Postgres>,
// {
//     fn into_sql(self, qb: &mut QueryBuilder<'_, Postgres>) {
//         qb.push_bind(self);
//     }
// }
