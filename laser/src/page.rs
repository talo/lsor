use serde::{Deserialize, Serialize};
use sqlx::{Postgres, QueryBuilder};

use crate::{
    cond::{and, gt, lt},
    ord::{AsOrderBy, ToOrderBy},
    select::{all, from},
    sql::{IntoSql, ToSql},
    value::{alias, concat},
};

use super::cursor::Cursor;

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct Pagination {
    pub cursor: Cursor,
    pub after: String,
    pub before: String,
    pub first: i32,
    pub last: i32,
}

pub fn select_page_items<O>(subquery: O, pagination: Pagination) -> SelectPageItems<O> {
    SelectPageItems {
        subquery,
        pagination,
    }
}

pub struct SelectPageItems<O> {
    pub subquery: O,
    pub pagination: Pagination,
}

impl<'args, O> ToSql<'args> for SelectPageItems<O>
where
    &'args O: IntoSql,
    &'args O::E: IntoSql,
    O: 'args + AsOrderBy,
{
    fn to_sql(&'args self, qb: &mut QueryBuilder<'args, Postgres>) {
        let order_by = self.subquery.as_order_by();
        let order_by_expr = order_by.expr;
        let order = order_by.order;
        let order_flipped = order.flip();

        from(
            from(
                from(&self.subquery, "page_items_inner")
                    .select(all())
                    .filter_by(and(
                        gt(
                            order_by_expr,
                            self.pagination.cursor.decode(&self.pagination.after),
                        ),
                        lt(
                            order_by_expr,
                            self.pagination.cursor.decode(&self.pagination.before),
                        ),
                    ))
                    .order_by(order_by_expr, order)
                    .limit(self.pagination.first),
                "page_items_outer",
            )
            .select(all())
            .order_by(order_by_expr, order_flipped)
            .limit(self.pagination.last),
            "page_items",
        )
        .select(concat(all(), alias(order_by_expr, "cursor")))
        .order_by(order_by_expr, order)
        .into_sql(qb);
    }
}

impl<O> IntoSql for SelectPageItems<O>
where
    for<'args> &'args O::E: IntoSql,
    O: ToOrderBy + IntoSql,
{
    fn into_sql(self, qb: &mut QueryBuilder<'_, Postgres>) {
        let order_by = self.subquery.to_order_by();
        let order_by_expr = order_by.expr;
        let order = order_by.order;
        let order_flipped = order.flip();

        from(
            from(
                from(self.subquery, "page_items_inner")
                    .select(all())
                    .filter_by(and(
                        gt(
                            &order_by_expr,
                            self.pagination.cursor.decode(&self.pagination.after),
                        ),
                        lt(
                            &order_by_expr,
                            self.pagination.cursor.decode(&self.pagination.before),
                        ),
                    ))
                    .order_by(&order_by_expr, order)
                    .limit(self.pagination.first),
                "page_items_outer",
            )
            .select(all())
            .order_by(&order_by_expr, order_flipped)
            .limit(self.pagination.last),
            "page_items",
        )
        .select(concat(all(), alias(&order_by_expr, "cursor")))
        .order_by(&order_by_expr, order)
        .into_sql(qb);
    }
}

pub fn select_page_info<O>(
    subquery: O,
    cursor: Cursor,
    start: String,
    end: String,
) -> SelectPageInfo<O> {
    SelectPageInfo {
        subquery,
        cursor,
        start,
        end,
    }
}

pub struct SelectPageInfo<O> {
    pub subquery: O,
    pub cursor: Cursor,
    pub start: String,
    pub end: String,
}

impl<'args, O> ToSql<'args> for SelectPageInfo<O>
where
    O: AsOrderBy + ToSql<'args>,
    O::E: 'args,
    &'args O::E: IntoSql,
{
    fn to_sql(&'args self, qb: &mut QueryBuilder<'args, Postgres>) {
        let after_cond = gt(
            self.subquery.as_order_by().expr,
            self.cursor.decode(&self.start),
        );
        let before_cond = lt(
            self.subquery.as_order_by().expr,
            self.cursor.decode(&self.end),
        );
        qb.push("SELECT COUNT(*) AS total_count, COUNT(CASE WHEN ");
        before_cond.into_sql(qb);
        qb.push(" THEN 1 END) > 0 AS has_prev_page, COUNT(CASE WHEN ");
        after_cond.into_sql(qb);
        qb.push(" THEN 1 END) > 0 AS has_next_page FROM (");
        self.subquery.to_sql(qb);
        qb.push(") AS page_info");
    }
}

impl<O> IntoSql for SelectPageInfo<O>
where
    O: AsOrderBy + IntoSql,
    O::E: IntoSql,
    for<'args> &'args O::E: IntoSql,
{
    fn into_sql(self, qb: &mut QueryBuilder<'_, Postgres>) {
        let after_cond = gt(
            self.subquery.as_order_by().expr,
            self.cursor.decode(&self.start),
        );
        let before_cond = lt(
            self.subquery.as_order_by().expr,
            self.cursor.decode(&self.end),
        );
        qb.push("SELECT COUNT(*) AS total_count, COUNT(CASE WHEN ");
        before_cond.into_sql(qb);
        qb.push(" THEN 1 END) > 0 AS has_prev_page, COUNT(CASE WHEN ");
        after_cond.into_sql(qb);
        qb.push(" THEN 1 END) > 0 AS has_next_page FROM (");
        self.subquery.into_sql(qb);
        qb.push(") AS page_info");
    }
}
