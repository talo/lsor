use async_graphql::{connection::Edge, OutputType};
use sqlx::{postgres::PgRow, Executor, FromRow, Postgres, QueryBuilder, Row as _};

use crate::{
    all, select_page_info, select_page_items,
    sql::{IntoSql, ToSql as _},
    upsert, Columns, Cursor, OrderBy, Pagination, Table, ToValues,
};

pub async fn save_one<'c, E, R>(executor: E, row: R) -> sqlx::Result<()>
where
    E: Executor<'c, Database = Postgres>,
    R: Columns + Table + ToValues,
{
    let mut qb = QueryBuilder::default();
    let stmt = upsert(row);
    stmt.to_sql(&mut qb);
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
) -> sqlx::Result<Option<R>>
where
    for<'args> &'args F: IntoSql,
    for<'args> &'args S: IntoSql,
    for<'r> R: FromRow<'r, PgRow>,
    E: Copy + Executor<'c, Database = Postgres>,
    S: Copy + IntoSql,
    R: OutputType + Table,
{
    let cursor = pagination.cursor;
    let subquery = R::table().select(all()).filter_by(filter).order_by(sort);

    let mut qb = QueryBuilder::default();
    select_page_items(&subquery, pagination).into_sql(&mut qb);
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

    let mut qb = QueryBuilder::default();
    select_page_info(
        &subquery,
        cursor,
        edges
            .first()
            .map(|edge| edge.cursor.clone())
            .unwrap_or(cursor.encode_min()),
        edges
            .last()
            .map(|edge| edge.cursor.clone())
            .unwrap_or(cursor.encode_max()),
    )
    .into_sql(&mut qb);
    qb.build()
        .fetch_optional(executor)
        .await
        .and_then(|row| row.as_ref().map(R::from_row).transpose())
}
