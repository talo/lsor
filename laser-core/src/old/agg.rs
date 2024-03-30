use sqlx::{Postgres, QueryBuilder};

use crate::sql::IntoSql;

pub fn avg<E>(expr: E) -> Avg<E> {
    Avg {
        expr: Box::new(expr),
    }
}

pub fn count<E>(expr: E) -> Count<E> {
    Count {
        expr: Box::new(expr),
    }
}

pub fn sum<E>(expr: E) -> Sum<E> {
    Sum {
        expr: Box::new(expr),
    }
}

#[derive(Clone, Debug)]
pub struct Avg<E> {
    pub expr: Box<E>,
}

impl<E> IntoSql for Avg<E>
where
    E: IntoSql,
{
    fn into_sql(self, qb: &mut QueryBuilder<'_, Postgres>) {
        qb.push("AVG(");
        self.expr.into_sql(qb);
        qb.push(")");
    }
}

#[derive(Clone, Debug)]
pub struct Count<E> {
    pub expr: Box<E>,
}

impl<E> IntoSql for Count<E>
where
    E: IntoSql,
{
    fn into_sql(self, qb: &mut QueryBuilder<'_, Postgres>) {
        qb.push("COUNT(");
        self.expr.into_sql(qb);
        qb.push(")");
    }
}

#[derive(Clone, Debug)]
pub struct Sum<E> {
    pub expr: Box<E>,
}

impl<E> IntoSql for Sum<E>
where
    E: IntoSql,
{
    fn into_sql(self, qb: &mut QueryBuilder<'_, Postgres>) {
        qb.push("SUM(");
        self.expr.into_sql(qb);
        qb.push(")");
    }
}
