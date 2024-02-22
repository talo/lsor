use std::fmt::Display;

use sqlx::{Postgres, QueryBuilder};

use crate::sql::{IntoSql, ToSql};

pub trait Column<'args>: ToSql<'args> + IntoSql {}

pub fn col(name: &str) -> ColumnName<&str> {
    ColumnName { name }
}

pub fn column(name: &str) -> ColumnName<&str> {
    ColumnName { name }
}

pub struct ColumnName<D> {
    pub name: D,
}

impl<'args, D> ToSql<'args> for ColumnName<D>
where
    D: Display,
{
    fn to_sql(&'args self, qb: &mut QueryBuilder<'args, Postgres>) {
        qb.push(&self.name);
    }
}

impl<D> IntoSql for ColumnName<D>
where
    D: Display,
{
    fn into_sql(self, qb: &mut QueryBuilder<'_, Postgres>) {
        qb.push(self.name);
    }
}

impl<D> IntoSql for &ColumnName<D>
where
    D: Copy + Display,
{
    fn into_sql(self, qb: &mut QueryBuilder<'_, Postgres>) {
        qb.push(self.name);
    }
}

impl<'args, D> Column<'args> for ColumnName<D> where D: Display {}

/// This trait is implemented by types that represent a set of columns in an SQL
/// table.
pub trait Columns {
    type D;

    fn columns() -> impl Iterator<Item = (ColumnName<Self::D>, bool)>;
}

impl<T> Columns for &T
where
    T: Columns,
{
    type D = T::D;

    fn columns() -> impl Iterator<Item = (ColumnName<Self::D>, bool)> {
        T::columns()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_column() {
        let col = column("id");
        let mut qb = QueryBuilder::default();
        col.to_sql(&mut qb);
        assert_eq!(qb.into_sql(), "id");
    }
}
