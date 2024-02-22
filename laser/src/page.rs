use serde::{Deserialize, Serialize};
use sqlx::{Postgres, QueryBuilder};

use crate::{
    cond::{and, gt, lt},
    select::{all, from},
    sql::{IntoSql, ToSql},
    value::{alias, concat},
};

use super::{cursor::Cursor, select::Ordered};

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct Pagination {
    pub cursor: Cursor,
    pub after: String,
    pub before: String,
    pub first: i32,
    pub last: i32,
}

pub fn select_page_items<S, E>(
    subquery: Ordered<S, E>,
    pagination: Pagination,
) -> SelectPageItems<S, E> {
    SelectPageItems {
        subquery,
        pagination,
    }
}

pub struct SelectPageItems<S, E> {
    pub subquery: Ordered<S, E>,
    pub pagination: Pagination,
}

impl<'args, S, E> ToSql<'args> for SelectPageItems<S, E>
where
    S: 'args,
    E: 'args,
    &'args E: IntoSql,
    &'args Ordered<S, E>: IntoSql,
{
    fn to_sql(&'args self, qb: &mut QueryBuilder<'args, Postgres>) {
        let cursor_expr = &self.subquery.cursor_expr;
        let order = self.subquery.order;
        let order_flipped = self.subquery.order.flip();
        from(
            from(
                from(&self.subquery, "page_items_inner")
                    .select(all())
                    .filter_by(and(
                        gt(
                            cursor_expr,
                            self.pagination.cursor.decode(&self.pagination.after),
                        ),
                        lt(
                            cursor_expr,
                            self.pagination.cursor.decode(&self.pagination.before),
                        ),
                    ))
                    .order_by(cursor_expr, order)
                    .limit(self.pagination.first),
                "page_items_outer",
            )
            .select(all())
            .order_by(cursor_expr, order_flipped)
            .limit(self.pagination.last),
            "page_items",
        )
        .select(concat(all(), alias(cursor_expr, "cursor")))
        .order_by(cursor_expr, order)
        .into_sql(qb);
    }
}

impl<S, E> IntoSql for SelectPageItems<S, E>
where
    for<'args> &'args E: IntoSql,
    for<'args> &'args Ordered<S, E>: IntoSql,
{
    fn into_sql(self, qb: &mut QueryBuilder<'_, Postgres>) {
        let cursor_expr = &self.subquery.cursor_expr;
        let order = self.subquery.order;
        let order_flipped = self.subquery.order.flip();

        from(
            from(
                from(&self.subquery, "page_items_inner")
                    .select(all())
                    .filter_by(and(
                        gt(
                            cursor_expr,
                            self.pagination.cursor.decode(&self.pagination.after),
                        ),
                        lt(
                            cursor_expr,
                            self.pagination.cursor.decode(&self.pagination.before),
                        ),
                    ))
                    .order_by(cursor_expr, order)
                    .limit(self.pagination.first),
                "page_items_outer",
            )
            .select(all())
            .order_by(cursor_expr, order_flipped)
            .limit(self.pagination.last),
            "page_items",
        )
        .select(concat(all(), alias(cursor_expr, "cursor")))
        .order_by(cursor_expr, order)
        .into_sql(qb);
    }
}

pub fn select_page_info<S, E>(
    subquery: Ordered<S, E>,
    cursor: Cursor,
    start: String,
    end: String,
) -> SelectPageInfo<S, E> {
    SelectPageInfo {
        subquery,
        cursor,
        start,
        end,
    }
}

pub struct SelectPageInfo<S, E> {
    pub subquery: Ordered<S, E>,
    pub cursor: Cursor,
    pub start: String,
    pub end: String,
}

impl<'args, S, E> ToSql<'args> for SelectPageInfo<S, E>
where
    E: 'args,
    &'args E: IntoSql,
    Ordered<S, E>: ToSql<'args>,
{
    fn to_sql(&'args self, qb: &mut QueryBuilder<'args, Postgres>) {
        let after_cond = gt(&self.subquery.cursor_expr, self.cursor.decode(&self.start));
        let before_cond = lt(&self.subquery.cursor_expr, self.cursor.decode(&self.end));
        qb.push("SELECT COUNT(*) AS total_count, COUNT(CASE WHEN ");
        before_cond.into_sql(qb);
        qb.push(" THEN 1 END) > 0 AS has_prev_page, COUNT(CASE WHEN ");
        after_cond.into_sql(qb);
        qb.push(" THEN 1 END) > 0 AS has_next_page FROM (");
        self.subquery.to_sql(qb);
        qb.push(") AS page_info");
    }
}

impl<S, E> IntoSql for SelectPageInfo<S, E>
where
    for<'args> &'args E: IntoSql,
    Ordered<S, E>: IntoSql,
{
    fn into_sql(self, qb: &mut QueryBuilder<'_, Postgres>) {
        let after_cond = gt(&self.subquery.cursor_expr, self.cursor.decode(&self.start));
        let before_cond = lt(&self.subquery.cursor_expr, self.cursor.decode(&self.end));
        qb.push("SELECT COUNT(*) AS total_count, COUNT(CASE WHEN ");
        before_cond.into_sql(qb);
        qb.push(" THEN 1 END) > 0 AS has_prev_page, COUNT(CASE WHEN ");
        after_cond.into_sql(qb);
        qb.push(" THEN 1 END) > 0 AS has_next_page FROM (");
        self.subquery.into_sql(qb);
        qb.push(") AS page_info");
    }
}
