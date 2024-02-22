use sqlx::{Encode, Postgres, QueryBuilder, Type};

pub trait ToSql<'args> {
    fn to_sql(&'args self, qb: &mut QueryBuilder<'args, Postgres>);
}

impl<'args, T> ToSql<'args> for T
where
    &'args T: 'args + Encode<'args, Postgres> + Send + Type<Postgres>,
{
    fn to_sql(&'args self, qb: &mut QueryBuilder<'args, Postgres>) {
        qb.push_bind(self);
    }
}

pub trait IntoSql {
    fn into_sql(self, qb: &mut QueryBuilder<'_, Postgres>);
}

impl<T> IntoSql for T
where
    for<'args> T: 'args + Encode<'args, Postgres> + Send + Type<Postgres>,
{
    fn into_sql(self, qb: &mut QueryBuilder<'_, Postgres>) {
        qb.push_bind(self);
    }
}
