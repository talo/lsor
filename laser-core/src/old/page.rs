use async_graphql::SimpleObject;
use serde::{Deserialize, Serialize};
use sqlx::{Postgres, QueryBuilder};

use crate::{
    cond::{and, gt, lt},
    cursor::Cursor,
    ord::ToOrderBy,
    select::{all, from},
    sql::IntoSql,
    value::{alias, concat},
    OrderBy,
};

#[derive(
    Clone, Copy, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize, SimpleObject,
)]
pub struct TotalCount {
    pub total_count: i64,
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct Pagination {
    pub cursor: Cursor,
    pub after: Option<String>,
    pub before: Option<String>,
    pub first: i32,
    pub last: i32,
}

pub fn select_page_items<O>(subquery: O, pagination: Pagination) -> SelectPageItems<O> {
    SelectPageItems {
        subquery,
        pagination,
    }
}

#[derive(Clone)]
pub struct SelectPageItems<O> {
    pub subquery: O,
    pub pagination: Pagination,
}

impl<O> IntoSql for SelectPageItems<O>
where
    O: ToOrderBy + IntoSql,
    O::By: Clone + IntoSql,
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
                            order_by_expr.clone(),
                            self.pagination
                                .after
                                .map(|v| self.pagination.cursor.decode(&v))
                                .unwrap_or(self.pagination.cursor.min()),
                        ),
                        lt(
                            order_by_expr.clone(),
                            self.pagination
                                .before
                                .map(|v| self.pagination.cursor.decode(&v))
                                .unwrap_or(self.pagination.cursor.max()),
                        ),
                    ))
                    .order_by(OrderBy {
                        expr: order_by_expr.clone(),
                        order,
                    })
                    .limit(self.pagination.first),
                "page_items_outer",
            )
            .select(all())
            .order_by(OrderBy {
                expr: order_by_expr.clone(),
                order: order_flipped,
            })
            .limit(self.pagination.last),
            "page_items",
        )
        .select(concat(all(), alias(order_by_expr.clone(), "cursor")))
        .order_by(OrderBy {
            expr: order_by_expr,
            order,
        })
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

#[derive(Clone)]
pub struct SelectPageInfo<O> {
    pub subquery: O,
    pub cursor: Cursor,
    pub start: String,
    pub end: String,
}

impl<O> IntoSql for SelectPageInfo<O>
where
    O: ToOrderBy + IntoSql,
    O::By: Clone + IntoSql,
{
    fn into_sql(self, qb: &mut QueryBuilder<'_, Postgres>) {
        let order_by = self.subquery.to_order_by();
        let after_cond = gt(order_by.expr.clone(), self.cursor.decode(&self.start));
        let before_cond = lt(order_by.expr, self.cursor.decode(&self.end));
        qb.push("SELECT COUNT(*) AS total_count, COUNT(CASE WHEN ");
        before_cond.into_sql(qb);
        qb.push(" THEN 1 END) > 0 AS has_prev_page, COUNT(CASE WHEN ");
        after_cond.into_sql(qb);
        qb.push(" THEN 1 END) > 0 AS has_next_page FROM (");
        self.subquery.into_sql(qb);
        qb.push(") AS page_info");
    }
}
