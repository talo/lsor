use sqlx::{Postgres, QueryBuilder};

use crate::{
    column::Columns,
    sql::IntoSql,
    table::{Table, TableName},
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

impl<R> IntoSql for Upsert<R>
where
    R: Columns,
{
    fn into_sql(self, qb: &mut QueryBuilder<'_, Postgres>) {
        qb.push("INSERT INTO ");
        self.table_name.into_sql(qb);
        qb.push(" (");
        for (i, (column_name, _)) in R::columns().enumerate() {
            if i > 0 {
                qb.push(", ");
            }
            column_name.into_sql(qb);
        }
        qb.push(") VALUES (");
        self.row.into_column_values(qb);
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
