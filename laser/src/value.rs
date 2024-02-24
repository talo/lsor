use sqlx::{Postgres, QueryBuilder};

use crate::sql::IntoSql;

pub fn alias<E>(expr: E, alias: &'static str) -> Alias<E> {
    Alias {
        expr,
        alias: alias.into(),
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Alias<E> {
    expr: E,
    alias: &'static str,
}

impl<E> IntoSql for Alias<E>
where
    E: IntoSql,
{
    fn into_sql(self, qb: &mut QueryBuilder<'_, Postgres>) {
        self.expr.into_sql(qb);
        qb.push(" AS ");
        qb.push(self.alias);
    }
}

pub fn concat<Head, Tail>(head: Head, tail: Tail) -> Concat<Head, Tail> {
    Concat { head, tail }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Concat<Head, Tail> {
    head: Head,
    tail: Tail,
}

impl<Head, Tail> IntoSql for Concat<Head, Tail>
where
    Head: IntoSql,
    Tail: IntoSql,
{
    fn into_sql(self, qb: &mut QueryBuilder<'_, Postgres>) {
        self.head.into_sql(qb);
        qb.push(", ");
        self.tail.into_sql(qb);
    }
}
