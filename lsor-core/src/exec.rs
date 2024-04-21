use async_graphql::{
    connection::{Connection, Edge, PageInfo},
    OutputType,
};
use sqlx::{postgres::PgRow, Executor, FromRow, Postgres};

use crate::{
    cursor::Cursor,
    driver::{Driver, PushPrql},
    from::from,
    page::{select_page_info, select_page_items, Pagination, TotalCount},
    row::{upsert, Row},
    table::Table,
    Sorting,
};

pub async fn save_one<'c, E, R>(executor: E, row: R) -> sqlx::Result<()>
where
    E: Executor<'c, Database = Postgres>,
    R: Row + Table,
{
    let mut driver = Driver::new();
    upsert(row).push_to_driver(&mut driver);
    driver.execute_without_compilation(executor).await?;
    Ok(())
}

pub async fn load_one<'c, E, F, R>(executor: E, filter: F) -> sqlx::Result<Option<R>>
where
    E: Executor<'c, Database = Postgres>,
    F: PushPrql,
    for<'r> R: FromRow<'r, PgRow> + Table,
{
    let mut driver = Driver::new();

    from(R::table_name())
        .filter(filter)
        .take(1)
        .push_to_driver(&mut driver);

    driver
        .fetch_optional(executor)
        .await
        .and_then(|row| row.as_ref().map(R::from_row).transpose())
}

pub async fn load_page<'c, E, F, S, R>(
    executor: E,
    filter: F,
    sort: S,
    pagination: Pagination,
) -> sqlx::Result<Connection<String, R, TotalCount>>
where
    E: Copy + Executor<'c, Database = Postgres>,
    F: PushPrql,
    S: PushPrql + Sorting,
    for<'r> R: FromRow<'r, PgRow> + OutputType + Table,
{
    use sqlx::Row;

    let cursor = pagination.cursor;
    let subquery = from(R::table_name()).filter(filter);
    let subquery = subquery.sort(sort);

    let mut driver = Driver::new();
    select_page_items(&subquery, pagination).push_to_driver(&mut driver);

    let rows = driver.fetch_all(executor).await?;
    let edges = rows
        .into_iter()
        .map(|row| {
            Ok(Edge::new(
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

    let mut driver = Driver::new();
    select_page_info(subquery, cursor, start.clone(), end.clone()).push_to_driver(&mut driver);
    let row = driver.fetch_one(executor).await?;
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
