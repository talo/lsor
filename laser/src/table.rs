use std::fmt::Display;

use sqlx::{Postgres, QueryBuilder};

use crate::{
    column::{Column, ColumnName},
    select::Select,
    sql::{IntoSql, ToSql},
};

pub trait Table {
    type D;

    fn table() -> TableName<Self::D>;
}

impl<T> Table for &T
where
    T: Table,
{
    type D = T::D;

    fn table() -> TableName<Self::D> {
        T::table()
    }
}

pub fn table(name: &str) -> TableName<&str> {
    TableName { name }
}

pub struct TableName<D> {
    pub name: D,
}

impl<D> TableName<D> {
    pub fn column<C>(self, name: C) -> TableColumnName<D, C> {
        TableColumnName {
            table: self,
            column: ColumnName { name },
        }
    }

    pub fn to_column<C>(&self, name: C) -> TableColumnName<&D, C> {
        TableColumnName {
            table: TableName { name: &self.name },
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

impl<'args, D> ToSql<'args> for TableName<D>
where
    D: Display,
{
    fn to_sql(&'args self, qb: &mut QueryBuilder<'args, Postgres>) {
        qb.push(&self.name);
    }
}

impl<D> IntoSql for TableName<D>
where
    D: Display,
{
    fn into_sql(self, qb: &mut QueryBuilder<'_, Postgres>) {
        qb.push(self.name);
    }
}

impl<'args, D> ToSql<'args> for &'args TableName<D>
where
    D: Display,
{
    fn to_sql(&'args self, qb: &mut QueryBuilder<'args, Postgres>) {
        qb.push(&self.name);
    }
}

impl<D> IntoSql for &TableName<D>
where
    D: Copy + Display,
{
    fn into_sql(self, qb: &mut QueryBuilder<'_, Postgres>) {
        qb.push(self.name);
    }
}

pub struct TableColumnName<T, C> {
    pub table: TableName<T>,
    pub column: ColumnName<C>,
}

impl<'args, T, C> ToSql<'args> for TableColumnName<T, C>
where
    T: Display,
    C: Display,
{
    fn to_sql(&'args self, qb: &mut QueryBuilder<'args, Postgres>) {
        qb.push("(");
        self.table.to_sql(qb);
        qb.push(".");
        self.column.to_sql(qb);
        qb.push(")");
    }
}

impl<T, C> IntoSql for TableColumnName<T, C>
where
    T: Display,
    C: Display,
{
    fn into_sql(self, qb: &mut QueryBuilder<'_, Postgres>) {
        qb.push("(");
        self.table.into_sql(qb);
        qb.push(".");
        self.column.into_sql(qb);
        qb.push(")");
    }
}

impl<'args, T, C> Column<'args> for TableColumnName<T, C>
where
    T: Display,
    C: Display,
{
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
        let tab = table("users");
        let tab_col = tab.to_column("id");
        let mut qb = QueryBuilder::default();
        tab_col.to_sql(&mut qb);
        assert_eq!(qb.into_sql(), "(users.id)");

        let tab_col = table("users").column("id");
        let mut qb = QueryBuilder::default();
        tab_col.to_sql(&mut qb);
        assert_eq!(qb.into_sql(), "(users.id)");
    }
}
