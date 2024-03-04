use async_graphql::{
    connection::{Connection, Edge, PageInfo},
    OutputType,
};
use sqlx::{postgres::PgRow, Executor, FromRow, Postgres, QueryBuilder, Row as _};

use crate::{
    all, select_page_info, select_page_items, sql::IntoSql, upsert, Columns, Cursor, OrderBy,
    Pagination, Table, TotalCount,
};

pub async fn save_one<'c, E, R>(executor: E, row: R) -> sqlx::Result<()>
where
    E: Executor<'c, Database = Postgres>,
    R: Columns + Table,
{
    let mut qb = QueryBuilder::default();
    upsert(row).into_sql(&mut qb);
    qb.build().execute(executor).await?;
    Ok(())
}

pub async fn load_one<'c, E, F, R>(executor: E, filter: F) -> sqlx::Result<Option<R>>
where
    E: Executor<'c, Database = Postgres>,
    F: IntoSql,
    for<'r> R: FromRow<'r, PgRow> + Table,
{
    let mut qb = QueryBuilder::default();

    R::table()
        .select(all())
        .filter_by(filter)
        .limit(1)
        .into_sql(&mut qb);

    qb.build()
        .fetch_optional(executor)
        .await
        .and_then(|row| row.as_ref().map(R::from_row).transpose())
}

pub async fn load_page<'c, E, F, S, R>(
    executor: E,
    filter: F,
    sort: OrderBy<S>,
    pagination: Pagination,
) -> sqlx::Result<Connection<String, R, TotalCount>>
where
    E: Copy + Executor<'c, Database = Postgres>,
    F: Clone + IntoSql,
    S: Clone + IntoSql,
    for<'r> R: FromRow<'r, PgRow> + OutputType + Table,
{
    let cursor = pagination.cursor;
    let subquery = R::table().select(all()).filter_by(filter).order_by(sort);

    let mut qb = QueryBuilder::default();
    select_page_items(subquery.clone(), pagination).into_sql(&mut qb);
    let rows = qb.build().fetch_all(executor).await?;
    let edges = rows
        .into_iter()
        .map(|row| {
            Ok(Edge::<_, _, _>::new(
                Cursor::infer(row.try_get_raw("cursor")?)?,
                R::from_row(&row)?,
            ))
        })
        .collect::<sqlx::Result<Vec<_>>>()?;

    let start = edges
        .first()
        .map(|edge| edge.cursor.clone())
        .unwrap_or(Cursor::encode(&cursor.min()));
    let end = edges
        .last()
        .map(|edge| edge.cursor.clone())
        .unwrap_or(Cursor::encode(&cursor.max()));
    let mut qb = QueryBuilder::default();
    select_page_info(subquery, cursor, start.clone(), end.clone()).into_sql(&mut qb);
    let row = qb.build().fetch_one(executor).await?;
    let page_info = PageInfo {
        has_next_page: row.try_get("has_next_page")?,
        has_previous_page: row.try_get("has_prev_page")?,
        start_cursor: Some(start),
        end_cursor: Some(end),
    };
    let total_count = TotalCount {
        total_count: row.try_get("total_count")?,
    };

    let mut conn = Connection::with_additional_fields(
        page_info.has_previous_page,
        page_info.has_next_page,
        total_count,
    );
    conn.edges = edges;
    Ok(conn)
}
