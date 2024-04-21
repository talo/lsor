use crate::{
    column::ColumnName,
    driver::{Driver, PushPrql},
    table::{Table, TableName},
};

pub type IsPk = bool;

pub trait Row {
    fn column_names() -> impl Iterator<Item = (ColumnName, IsPk)>;
    fn push_column_values(&self, driver: &mut Driver);
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
        // TODO: For now, the PRQL compiler is unable to handle insert
        // statements. To work around this, we inject raw SQL into the driver
        // (and later - during execution - we directly execute the PRQL without
        // compilation).
        //
        // As such, we need to ensure that the driver is empty before we push
        // anything into it, because raw SQL is obvious not compatible with
        // PRQL. We have tried using the S-string in PRQL but that also does not
        // work, because all PRQL expressions must start with select statements.
        //
        // This is a temporary solution until the PRQL compiler is able to
        // handle insert statements.
        assert!(driver.is_empty());

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
        self.row.push_column_values(driver);
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
        driver.push(")");
    }
}
