use sqlx::{Postgres, QueryBuilder};

use crate::{
    sql::{IntoSql, ToSql},
    Order, OrderBy,
};

pub fn col(name: &'static str) -> ColumnName {
    ColumnName { name }
}

pub fn column(name: &'static str) -> ColumnName {
    ColumnName { name }
}

#[derive(Copy, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
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

impl<'args> ToSql<'args> for ColumnName {
    fn to_sql(&'args self, qb: &mut QueryBuilder<'args, Postgres>) {
        qb.push(&self.name);
    }
}

impl IntoSql for ColumnName {
    fn into_sql(self, qb: &mut QueryBuilder<'_, Postgres>) {
        qb.push(self.name);
    }
}

impl IntoSql for &ColumnName {
    fn into_sql(self, qb: &mut QueryBuilder<'_, Postgres>) {
        qb.push(self.name);
    }
}

/// This trait is implemented by types that represent a set of columns in an SQL
/// table.
pub trait Columns {
    fn columns() -> impl Iterator<Item = (ColumnName, bool)>;
}

impl<T> Columns for &T
where
    T: Columns,
{
    fn columns() -> impl Iterator<Item = (ColumnName, bool)> {
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
