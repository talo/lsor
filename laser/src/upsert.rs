use sqlx::{Postgres, QueryBuilder};

use crate::{
    column::Columns,
    sql::{IntoSql, ToSql},
    table::{Table, TableName},
    value::ToValues,
};

pub struct Upsert<R> {
    pub table_name: TableName,
    pub row: R,
}

pub fn upsert<R>(row: R) -> Upsert<R>
where
    R: Table,
{
    Upsert {
        table_name: R::table(),
        row,
    }
}

pub fn upsert_into<R>(table_name: TableName, row: R) -> Upsert<R> {
    Upsert { table_name, row }
}

impl<'args, R> ToSql<'args> for Upsert<R>
where
    R: Columns + ToValues,
{
    fn to_sql(&'args self, qb: &mut QueryBuilder<'args, Postgres>) {
        qb.push("INSERT INTO ");
        self.table_name.to_sql(qb);
        qb.push(" (");
        for (i, (column_name, _)) in R::columns().enumerate() {
            if i > 0 {
                qb.push(", ");
            }
            column_name.into_sql(qb);
        }
        qb.push(") VALUES (");
        for (i, column_value) in self.row.to_values().enumerate() {
            if i > 0 {
                qb.push(", ");
            }
            column_value.to_sql(qb);
        }
        qb.push(") ON CONFLICT (");
        for (i, (column_name, _)) in R::columns().filter(|(_, pk)| *pk).enumerate() {
            if i > 0 {
                qb.push(", ");
            }
            column_name.into_sql(qb);
        }
        qb.push(") DO UPDATE SET (");
        for (i, (column_name, _)) in R::columns().filter(|(_, pk)| !*pk).enumerate() {
            if i > 0 {
                qb.push(", ");
            }
            column_name.into_sql(qb);
        }
        qb.push(") = (");
        for (i, (j, _)) in R::columns()
            .enumerate()
            .filter(|(_, (_, pk))| !*pk)
            .enumerate()
        {
            if i > 0 {
                qb.push(", ");
            }
            qb.push("$");
            qb.push(j + 1);
        }
        qb.push(")");
    }
}
