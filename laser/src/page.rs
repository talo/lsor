use async_graphql::SimpleObject;
use serde::{Deserialize, Serialize};

use crate::{
    aggregate::Aggregate,
    column::col,
    cond::{and, gt, lt},
    cursor::Cursor,
    driver::PushPrql,
    either::if_then_else,
    expr::{case, count, sum, when},
    filter::Filtered,
    sort::SortedBy,
    var::{one, zero},
};

#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    Deserialize,
    Eq,
    Hash,
    PartialEq,
    PartialOrd,
    Ord,
    Serialize,
    SimpleObject,
)]
#[graphql(rename_fields = "snake_case")]
pub struct TotalCount {
    pub total_count: i64,
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct Pagination {
    pub cursor: Cursor,
    pub after: Option<String>,
    pub before: Option<String>,
    pub first: usize,
    pub last: usize,
}

pub fn select_page_info<Query>(
    query: Query,
    cursor: Cursor,
    start: String,
    end: String,
) -> SelectPageInfo<Query> {
    SelectPageInfo {
        query,
        cursor,
        start,
        end,
    }
}

pub fn select_page_items<Query>(query: Query, pagination: Pagination) -> SelectPageItems<Query> {
    SelectPageItems { query, pagination }
}

pub struct SelectPageInfo<Query> {
    pub query: Query,
    pub cursor: Cursor,
    pub start: String,
    pub end: String,
}

impl<Query> PushPrql for SelectPageInfo<Query>
where
    Query: SortedBy + PushPrql,
    <Query as SortedBy>::By: PushPrql,
{
    fn push_to_driver(&self, driver: &mut crate::driver::Driver) {
        let sort = self.query.sorted_by();
        let start = self.cursor.decode(&self.start);
        let end = self.cursor.decode(&self.end);
        Aggregate {
            query: &self.query,
            aggregations: vec![
                (col("total_count"), &count() as &dyn PushPrql),
                (
                    col("has_prev_page"),
                    &gt(
                        sum(case([when(if_then_else(
                            sort.is_asc(),
                            || lt(sort.by(), &start),
                            || gt(sort.by(), &start),
                        ))
                        .then(one())])),
                        zero(),
                    ) as &dyn PushPrql,
                ),
                (
                    col("has_next_page"),
                    &gt(
                        sum(case([when(if_then_else(
                            sort.is_asc(),
                            || gt(sort.by(), &end),
                            || lt(sort.by(), &end),
                        ))
                        .then(one())])),
                        zero(),
                    ) as &dyn PushPrql,
                ),
            ],
        }
        .push_to_driver(driver);
    }
}

pub struct SelectPageItems<Query> {
    pub query: Query,
    pub pagination: Pagination,
}

impl<Query> PushPrql for SelectPageItems<Query>
where
    Query: SortedBy + PushPrql,
    <Query as SortedBy>::By: PushPrql,
{
    fn push_to_driver(&self, driver: &mut crate::driver::Driver) {
        let sort = self.query.sorted_by();

        let after = self
            .pagination
            .after
            .as_ref()
            .map(|v| self.pagination.cursor.decode(v))
            .unwrap_or(self.pagination.cursor.min());

        let before = self
            .pagination
            .before
            .as_ref()
            .map(|v| self.pagination.cursor.decode(v))
            .unwrap_or(self.pagination.cursor.max());

        Filtered {
            query: &self.query,
            filter: and(
                if_then_else(
                    sort.is_asc(),
                    || gt(sort.by(), &after),
                    || lt(sort.by(), &after),
                ),
                if_then_else(
                    sort.is_asc(),
                    || lt(sort.by(), &before),
                    || gt(sort.by(), &before),
                ),
            ),
        }
        .take(self.pagination.first)
        .sort(sort.flip())
        .take(self.pagination.last)
        .sort(sort.as_ref())
        .push_to_driver(driver);
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{driver::Driver, from::from, table::table};

    #[test]
    fn test_select_page_info() {
        let mut driver = Driver::new();
        {
            let query = from(table("page"))
                .derive("cursor", col("created_at"))
                .sort(col("cursor").asc());
            let cursor = Cursor::String;
            let start = "start".to_string();
            let end = "end".to_string();
            let select_page_info = SelectPageInfo {
                query,
                cursor,
                start,
                end,
            };
            select_page_info.push_to_driver(&mut driver);
        }
        assert_eq!(driver.sql(), "WITH table_2 AS (SELECT COUNT(*) AS total_count, created_at AS _expr_0, COALESCE(SUM(CASE WHEN created_at < $1 THEN 1 ELSE NULL END), 0) AS _expr_2 FROM page), table_0 AS (SELECT total_count, _expr_2 > 0 AS has_prev_page, _expr_0, COALESCE(SUM(CASE WHEN _expr_0 > $2 THEN 1 ELSE NULL END), 0) AS _expr_1 FROM table_2), table_1 AS (SELECT total_count, has_prev_page, _expr_1 > 0 AS has_next_page, _expr_0 FROM table_0) SELECT total_count, has_prev_page, has_next_page FROM table_1");
    }

    #[test]
    fn test_select_page_items() {
        let mut driver = Driver::new();
        {
            let query = from(table("page"))
                .derive("cursor", col("created_at"))
                .sort(col("cursor").desc());
            let cursor = Cursor::String;
            let after = Some("after".to_string());
            let before = Some("before".to_string());
            let select_page_info = SelectPageItems {
                query,
                pagination: Pagination {
                    cursor,
                    after,
                    before,
                    first: 10,
                    last: 5,
                },
            };
            select_page_info.push_to_driver(&mut driver);
        }
        assert_eq!(driver.sql(), "WITH table_2 AS (SELECT *, created_at AS cursor FROM page), table_1 AS (SELECT * FROM table_2 WHERE cursor < $1 AND cursor > $2 ORDER BY cursor DESC LIMIT 10), table_0 AS (SELECT * FROM table_1 ORDER BY cursor LIMIT 5) SELECT * FROM table_0 ORDER BY cursor DESC");
    }
}
