use sqlx::{Postgres, QueryBuilder};

use crate::{column::ColumnName, select::Select, sql::IntoSql};

pub trait Table {
    fn table() -> TableName;
}

impl<T> Table for &T
where
    T: Table,
{
    fn table() -> TableName {
        T::table()
    }
}

pub fn table(name: &'static str) -> TableName {
    TableName { name }
}

#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq)]
pub struct TableName {
    pub name: &'static str,
}

impl TableName {
    pub fn column(self, name: &'static str) -> TableColumnName {
        TableColumnName {
            table: self,
            column: ColumnName { name },
        }
    }

    pub fn select<E>(self, expr: E) -> Select<E, Self> {
        Select {
            expr,
            from_items: self,
        }
    }
}

impl IntoSql for TableName {
    fn into_sql(self, qb: &mut QueryBuilder<'_, Postgres>) {
        qb.push(self.name);
    }
}

#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq)]
pub struct TableColumnName {
    pub table: TableName,
    pub column: ColumnName,
}

impl IntoSql for TableColumnName {
    fn into_sql(self, qb: &mut QueryBuilder<'_, Postgres>) {
        qb.push("(");
        self.table.into_sql(qb);
        qb.push(".");
        self.column.into_sql(qb);
        qb.push(")");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_table() {
        let mut qb = QueryBuilder::default();
        table("users").into_sql(&mut qb);
        assert_eq!(qb.into_sql(), "users");
    }

    #[test]
    fn test_table_column() {
        let mut qb = QueryBuilder::default();
        table("users").column("id").into_sql(&mut qb);
        assert_eq!(qb.into_sql(), "(users.id)");
    }
}
