use crate::{
    column::ColumnName,
    driver::{Driver, PushPrql},
    table::{Table, TableName},
};

pub type IsPk = bool;

pub trait Row {
    fn column_names() -> impl Iterator<Item = (ColumnName, IsPk)>;
    fn column_values(&self) -> impl Iterator<Item = (&dyn PushPrql, IsPk)>;
}

pub fn upsert<R>(row: R) -> Upsert<R>
where
    R: Table,
{
    Upsert {
        table_name: R::table_name(),
        row,
    }
}

pub fn upsert_into<R>(table_name: TableName, row: R) -> Upsert<R> {
    Upsert { table_name, row }
}

pub struct Upsert<R> {
    pub table_name: TableName,
    pub row: R,
}

impl<R> PushPrql for Upsert<R>
where
    R: Row,
{
    fn push_to_driver(&self, driver: &mut Driver) {
        driver.push("s\'");
        driver.push("INSERT INTO ");
        self.table_name.push_to_driver(driver);
        driver.push(" (");
        for (i, (column_name, _)) in R::column_names().enumerate() {
            if i > 0 {
                driver.push(", ");
            }
            column_name.push_to_driver(driver);
        }
        driver.push(") VALUES (");
        for (i, (column_value, _)) in self.row.column_values().enumerate() {
            if i > 0 {
                driver.push(", ");
            }
            column_value.push_to_driver(driver);
        }
        driver.push(") ON CONFLICT (");
        for (i, (column_name, _)) in R::column_names().filter(|(_, pk)| *pk).enumerate() {
            if i > 0 {
                driver.push(", ");
            }
            column_name.push_to_driver(driver);
        }
        driver.push(") DO UPDATE SET (");
        for (i, (column_name, _)) in R::column_names().filter(|(_, pk)| !*pk).enumerate() {
            if i > 0 {
                driver.push(", ");
            }
            column_name.push_to_driver(driver);
        }
        driver.push(") = (");
        for (i, (j, _)) in R::column_names()
            .enumerate()
            .filter(|(_, (_, pk))| !*pk)
            .enumerate()
        {
            if i > 0 {
                driver.push(", ");
            }
            driver.push("$");
            driver.push(j + 1);
        }
        driver.push(")\'");
    }
}
