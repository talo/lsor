use sqlx::{Postgres, QueryBuilder};

use crate::{sql::IntoSql, Order, OrderBy};

/// This trait is implemented by types that represent a set of columns in an SQL
/// table.
pub trait Columns {
    fn columns() -> impl Iterator<Item = (ColumnName, bool)>;
    fn into_column_values(self, qb: &mut QueryBuilder<'_, Postgres>);
}

impl<T> Columns for &T
where
    T: Copy + Columns,
{
    fn columns() -> impl Iterator<Item = (ColumnName, bool)> {
        T::columns()
    }

    fn into_column_values(self, qb: &mut QueryBuilder<'_, Postgres>) {
        (*self).into_column_values(qb);
    }
}

pub fn col(name: &'static str) -> ColumnName {
    ColumnName { name }
}

pub fn column(name: &'static str) -> ColumnName {
    ColumnName { name }
}

#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq)]
pub struct ColumnName {
    pub name: &'static str,
}

impl ColumnName {
    pub fn asc(self) -> OrderBy<Self> {
        OrderBy {
            expr: self,
            order: Order::Asc,
        }
    }

    pub fn desc(self) -> OrderBy<Self> {
        OrderBy {
            expr: self,
            order: Order::Desc,
        }
    }

    pub fn as_asc(&self) -> OrderBy<&Self> {
        OrderBy {
            expr: self,
            order: Order::Asc,
        }
    }

    pub fn as_desc(&self) -> OrderBy<&Self> {
        OrderBy {
            expr: self,
            order: Order::Desc,
        }
    }
}

impl IntoSql for ColumnName {
    fn into_sql(self, qb: &mut QueryBuilder<'_, Postgres>) {
        qb.push(self.name);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_column() {
        let mut qb = QueryBuilder::default();
        column("id").into_sql(&mut qb);
        assert_eq!(qb.into_sql(), "id");
    }
}
