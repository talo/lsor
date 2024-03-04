use sqlx::{Postgres, QueryBuilder};

pub trait IntoSql {
    fn into_sql(self, qb: &mut QueryBuilder<'_, Postgres>);
}

impl<T> IntoSql for T
where
    for<'args> T:
        'args + ::sqlx::Encode<'args, ::sqlx::Postgres> + Send + ::sqlx::Type<::sqlx::Postgres>,
{
    fn into_sql(self, qb: &mut QueryBuilder<'_, Postgres>) {
        qb.push_bind(self);
    }
}
