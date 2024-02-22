pub mod column;
pub mod cond;
pub mod cursor;
pub mod lit;
pub mod ord;
pub mod page;
pub mod select;
pub mod sql;
pub mod table;
pub mod upsert;
pub mod value;

#[cfg(test)]
mod test {
    use chrono::Utc;
    use sqlx::QueryBuilder;

    use crate::{
        column::col,
        cond::{eq, if_then_else},
        cursor::{Cursor, DateTimeCursor},
        ord::desc,
        page::{select_page_info, select_page_items, Pagination},
        select::all,
        sql::IntoSql,
        table::table,
    };

    #[test]
    pub fn conditional_select() {
        let mut qb = QueryBuilder::new("");
        table("foo")
            .select(if_then_else(true, all(), col("foo")))
            .filter_by(if_then_else(
                true,
                eq(col("id"), 1),
                eq(col("foo"), col("bar")),
            ))
            .order_by(col("id"), desc())
            .limit(10)
            .into_sql(&mut qb);
        assert_eq!(
            qb.into_sql(),
            "SELECT * FROM foo WHERE (id) = ($1) ORDER BY id DESC LIMIT $2"
        );
    }

    #[test]
    pub fn conditional_filter_by() {
        let mut qb = QueryBuilder::new("");
        table("foo")
            .select(all())
            .filter_by(if_then_else(
                true,
                eq(col("id"), 1),
                eq(col("foo"), col("bar")),
            ))
            .order_by(col("id"), desc())
            .limit(10)
            .into_sql(&mut qb);
        assert_eq!(
            qb.into_sql(),
            "SELECT * FROM foo WHERE (id) = ($1) ORDER BY id DESC LIMIT $2"
        );

        let mut qb = QueryBuilder::new("");
        table("foo")
            .select(all())
            .filter_by(if_then_else(
                false,
                eq(col("id"), 1),
                eq(col("foo"), col("bar")),
            ))
            .order_by(col("id"), desc())
            .limit(10)
            .into_sql(&mut qb);
        assert_eq!(
            qb.into_sql(),
            "SELECT * FROM foo WHERE (foo) = (bar) ORDER BY id DESC LIMIT $1"
        );
    }

    #[test]
    pub fn select_from_where_eq_order_by_limit() {
        let mut qb = QueryBuilder::new("");
        table("foo")
            .select(all())
            .filter_by(eq(col("id"), 1))
            .order_by(col("id"), desc())
            .limit(10)
            .into_sql(&mut qb);
        assert_eq!(
            qb.into_sql(),
            "SELECT * FROM foo WHERE (id) = ($1) ORDER BY id DESC LIMIT $2"
        );
    }

    #[test]
    pub fn select_page() {
        let after = DateTimeCursor::encode(Utc::now());
        let before = DateTimeCursor::encode(Utc::now());
        let first = 10;
        let last = 5;

        let mut qb = QueryBuilder::new("");
        select_page_items(
            table("entities").select(all()).order_by(col("id"), desc()),
            Pagination {
                after,
                before,
                first,
                last,
                cursor: Cursor::DateTime,
            },
        )
        .into_sql(&mut qb);
        assert_eq!(qb.into_sql(), "SELECT *, id AS cursor FROM (SELECT * FROM (SELECT * FROM (SELECT * FROM entities ORDER BY id DESC) AS page_items_inner WHERE ((id) > ($1)) AND ((id) < ($2)) ORDER BY id DESC LIMIT $3) AS page_items_outer ORDER BY id ASC LIMIT $4) AS page_items ORDER BY id DESC");

        let mut qb = QueryBuilder::new("");
        select_page_info(
            table("entities").select(all()).order_by(col("id"), desc()),
            Cursor::DateTime,
            DateTimeCursor::encode(Utc::now()),
            DateTimeCursor::encode(Utc::now()),
        )
        .into_sql(&mut qb);
        assert_eq!(qb.into_sql(), "SELECT COUNT(*) AS total_count, COUNT(CASE WHEN (id) < ($1) THEN 1 END) > 0 AS has_prev_page, COUNT(CASE WHEN (id) > ($2) THEN 1 END) > 0 AS has_next_page FROM (SELECT * FROM entities ORDER BY id DESC) AS page_info");
    }
}
