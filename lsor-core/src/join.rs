use crate::{PushPrql, TableName};

pub fn inner_join<Expr, Cond>(expr: Expr, table: TableName, cond: Cond) -> Join<Expr, Cond> {
    Join {
        expr,
        side: Side::Inner,
        table,
        cond,
    }
}

pub fn left_join<Expr, Cond>(expr: Expr, table: TableName, cond: Cond) -> Join<Expr, Cond> {
    Join {
        expr,
        side: Side::Left,
        table,
        cond,
    }
}

pub fn right_join<Expr, Cond>(expr: Expr, table: TableName, cond: Cond) -> Join<Expr, Cond> {
    Join {
        expr,
        side: Side::Right,
        table,
        cond,
    }
}

pub fn full_join<Expr, Cond>(expr: Expr, table: TableName, cond: Cond) -> Join<Expr, Cond> {
    Join {
        expr,
        side: Side::Full,
        table,
        cond,
    }
}

pub enum Side {
    Inner,
    Left,
    Right,
    Full,
}

pub struct Join<Expr, Cond> {
    pub expr: Expr,
    pub side: Side,
    pub table: TableName,
    pub cond: Cond,
}

impl<Expr, Cond> PushPrql for Join<Expr, Cond>
where
    Expr: PushPrql,
    Cond: PushPrql,
{
    fn push_to_driver(&self, driver: &mut crate::driver::Driver) {
        self.expr.push_to_driver(driver);
        match self.side {
            Side::Inner => driver.push("\njoin side:inner "),
            Side::Left => driver.push("\njoin side:left "),
            Side::Right => driver.push("\njoin side:right "),
            Side::Full => driver.push("\njoin side:full "),
        }
        self.table.push_to_driver(driver);
        driver.push(" (");
        self.cond.push_to_driver(driver);
        driver.push(')');
    }
}

#[cfg(test)]
mod test {
    use crate::{column::col, eq, from::from, table::table};

    use super::*;

    #[test]
    fn test_left_join() {
        let mut driver = crate::driver::Driver::new();
        left_join(
            from(table("users")),
            table("tokens"),
            eq(
                table("users").dot(col("id")),
                table("tokens").dot(col("user_id")),
            ),
        )
        .push_to_driver(&mut driver);
        assert_eq!(
            driver.prql(),
            "from users\njoin side:left tokens ((users.id) == (tokens.user_id))"
        );
        assert_eq!(
            driver.sql(),
            "SELECT users.*, tokens.* FROM users LEFT JOIN tokens ON users.id = tokens.user_id"
        );
    }
}
