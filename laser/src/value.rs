use sqlx::{Encode, Postgres, QueryBuilder, Type};

use crate::sql::{IntoSql, ToSql};

/// This trait is implemented by types that represent a set of values from an
/// SQL table.
pub trait ToValues {
    fn to_values(&self) -> impl Iterator<Item = &dyn ToSql>;
}

impl<T> ToValues for &T
where
    T: ToValues,
{
    fn to_values(&self) -> impl Iterator<Item = &dyn ToSql> {
        (*self).to_values()
    }
}

pub fn var<V>(v: V) -> Var<V> {
    Var { v }
}

pub struct Var<V> {
    pub v: V,
}

impl<'args, V> ToSql<'args> for Var<V>
where
    &'args V: 'args + Encode<'args, Postgres> + Send + Type<Postgres>,
{
    fn to_sql(&'args self, qb: &mut QueryBuilder<'args, Postgres>) {
        qb.push_bind(&self.v);
    }
}

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

impl<'args, E> ToSql<'args> for Alias<E>
where
    E: ToSql<'args>,
{
    fn to_sql(&'args self, qb: &mut QueryBuilder<'args, Postgres>) {
        self.expr.to_sql(qb);
        qb.push(" AS ");
        qb.push(self.alias);
    }
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

impl<'args, Head, Tail> ToSql<'args> for Concat<Head, Tail>
where
    Head: ToSql<'args>,
    Tail: ToSql<'args>,
{
    fn to_sql(&'args self, qb: &mut QueryBuilder<'args, Postgres>) {
        self.head.to_sql(qb);
        qb.push(", ");
        self.tail.to_sql(qb);
    }
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
