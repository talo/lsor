use sqlx::{Postgres, QueryBuilder};

use crate::{
    column::ColumnName,
    select::Select,
    sql::{IntoSql, ToSql},
};

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

#[derive(Copy, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
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

impl<'args> ToSql<'args> for TableName {
    fn to_sql(&'args self, qb: &mut QueryBuilder<'args, Postgres>) {
        qb.push(&self.name);
    }
}

impl IntoSql for TableName {
    fn into_sql(self, qb: &mut QueryBuilder<'_, Postgres>) {
        qb.push(self.name);
    }
}

impl<'args> ToSql<'args> for &'args TableName {
    fn to_sql(&'args self, qb: &mut QueryBuilder<'args, Postgres>) {
        qb.push(&self.name);
    }
}

impl IntoSql for &TableName {
    fn into_sql(self, qb: &mut QueryBuilder<'_, Postgres>) {
        qb.push(self.name);
    }
}

pub struct TableColumnName {
    pub table: TableName,
    pub column: ColumnName,
}

impl<'args> ToSql<'args> for TableColumnName {
    fn to_sql(&'args self, qb: &mut QueryBuilder<'args, Postgres>) {
        qb.push("(");
        self.table.to_sql(qb);
        qb.push(".");
        self.column.to_sql(qb);
        qb.push(")");
    }
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
        let tab = table("users");
        let mut qb = QueryBuilder::default();
        tab.to_sql(&mut qb);
        assert_eq!(qb.into_sql(), "users");
    }

    #[test]
    fn test_table_column() {
        let tab_col = table("users").column("id");
        let mut qb = QueryBuilder::default();
        tab_col.to_sql(&mut qb);
        assert_eq!(qb.into_sql(), "(users.id)");
    }
}
