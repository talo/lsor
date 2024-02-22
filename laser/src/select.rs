use sqlx::{Postgres, QueryBuilder};

use crate::sql::IntoSql;

use super::{ord::Order, sql::ToSql};

pub fn from<T>(table_reference: T, alias: &'static str) -> FromClause<T> {
    FromClause {
        table_reference,
        alias,
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FromClause<T> {
    pub table_reference: T,
    pub alias: &'static str,
}

impl<T> FromClause<T> {
    pub fn select<E>(self, expr: E) -> Select<E, Self> {
        Select {
            expr,
            from_items: self,
        }
    }
}

impl<'args, T> ToSql<'args> for FromClause<T>
where
    T: ToSql<'args>,
{
    fn to_sql(&'args self, qb: &mut QueryBuilder<'args, Postgres>) {
        qb.push("(");
        self.table_reference.to_sql(qb);
        qb.push(") AS ");
        qb.push(self.alias);
    }
}

impl<T> IntoSql for FromClause<T>
where
    for<'args> T: IntoSql,
{
    fn into_sql(self, qb: &mut QueryBuilder<'_, Postgres>) {
        qb.push("(");
        self.table_reference.into_sql(qb);
        qb.push(") AS ");
        qb.push(self.alias);
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Select<ExprList, FromItems> {
    pub expr: ExprList,
    pub from_items: FromItems,
}

impl<ExprList, FromItems> Select<ExprList, FromItems> {
    pub fn filter_by<C>(self, condition: C) -> Filtered<Self, C> {
        Filtered {
            selection: self,
            condition,
        }
    }

    pub fn order_by<E>(self, expr: E, order: Order) -> Ordered<Self, E> {
        Ordered {
            selection: self,
            cursor_expr: expr,
            order,
        }
    }
}

impl<'args, E, FromItems> ToSql<'args> for Select<E, FromItems>
where
    E: ToSql<'args>,
    FromItems: ToSql<'args>,
{
    fn to_sql(&'args self, qb: &mut QueryBuilder<'args, Postgres>) {
        qb.push("SELECT ");
        self.expr.to_sql(qb);
        qb.push(" FROM ");
        self.from_items.to_sql(qb);
    }
}

impl<E, FromItems> IntoSql for Select<E, FromItems>
where
    E: IntoSql,
    FromItems: IntoSql,
{
    fn into_sql(self, qb: &mut QueryBuilder<'_, Postgres>) {
        qb.push("SELECT ");
        self.expr.into_sql(qb);
        qb.push(" FROM ");
        self.from_items.into_sql(qb);
    }
}

impl<E, FromItems> IntoSql for &Select<E, FromItems>
where
    for<'args> &'args E: IntoSql,
    for<'args> &'args FromItems: IntoSql,
{
    fn into_sql(self, qb: &mut QueryBuilder<'_, Postgres>) {
        qb.push("SELECT ");
        self.expr.into_sql(qb);
        qb.push(" FROM ");
        self.from_items.into_sql(qb);
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Filtered<S, C> {
    pub selection: S,
    pub condition: C,
}

impl<S, C> Filtered<S, C> {
    pub fn order_by<E>(self, expr: E, order: Order) -> Ordered<Self, E> {
        Ordered {
            selection: self,
            cursor_expr: expr,
            order,
        }
    }
}

impl<'args, S, C> ToSql<'args> for Filtered<S, C>
where
    S: ToSql<'args>,
    C: ToSql<'args>,
{
    fn to_sql(&'args self, qb: &mut QueryBuilder<'args, Postgres>) {
        self.selection.to_sql(qb);
        qb.push(" WHERE ");
        self.condition.to_sql(qb);
    }
}

impl<S, C> IntoSql for Filtered<S, C>
where
    for<'args> S: IntoSql,
    for<'args> C: IntoSql,
{
    fn into_sql(self, qb: &mut QueryBuilder<'_, Postgres>) {
        self.selection.into_sql(qb);
        qb.push(" WHERE ");
        self.condition.into_sql(qb);
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Ordered<S, E> {
    pub selection: S,
    pub cursor_expr: E,
    pub order: Order,
}

impl<S, E> Ordered<S, E> {
    pub fn limit(self, limit: i32) -> Limited<Self> {
        Limited {
            selection: self,
            limit,
        }
    }
}

impl<'args, S, E> ToSql<'args> for Ordered<S, E>
where
    S: ToSql<'args>,
    E: ToSql<'args>,
{
    fn to_sql(&'args self, qb: &mut QueryBuilder<'args, Postgres>) {
        self.selection.to_sql(qb);
        qb.push(" ORDER BY ");
        self.cursor_expr.to_sql(qb);
        qb.push(" ");
        self.order.to_sql(qb);
    }
}

impl<S, E> IntoSql for Ordered<S, E>
where
    S: IntoSql,
    E: IntoSql,
{
    fn into_sql(self, qb: &mut QueryBuilder<'_, Postgres>) {
        self.selection.into_sql(qb);
        qb.push(" ORDER BY ");
        self.cursor_expr.into_sql(qb);
        qb.push(" ");
        self.order.into_sql(qb);
    }
}

impl<S, E> IntoSql for &Ordered<S, E>
where
    for<'args> &'args S: IntoSql,
    for<'args> &'args E: IntoSql,
{
    fn into_sql(self, qb: &mut QueryBuilder<'_, Postgres>) {
        self.selection.into_sql(qb);
        qb.push(" ORDER BY ");
        self.cursor_expr.into_sql(qb);
        qb.push(" ");
        self.order.into_sql(qb);
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Limited<S> {
    pub selection: S,
    pub limit: i32,
}

impl<'args, S> ToSql<'args> for Limited<S>
where
    S: ToSql<'args>,
{
    fn to_sql(&'args self, qb: &mut QueryBuilder<'args, Postgres>) {
        self.selection.to_sql(qb);
        qb.push(" LIMIT ");
        qb.push_bind(self.limit);
    }
}

impl<S> IntoSql for Limited<S>
where
    for<'args> S: IntoSql,
{
    fn into_sql(self, qb: &mut QueryBuilder<'_, Postgres>) {
        self.selection.into_sql(qb);
        qb.push(" LIMIT ");
        qb.push_bind(self.limit);
    }
}

pub fn all() -> All {
    All
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct All;

impl<'args> ToSql<'args> for All {
    fn to_sql(&'args self, qb: &mut QueryBuilder<'args, Postgres>) {
        qb.push("*");
    }
}

impl<'args> ToSql<'args> for &All {
    fn to_sql(&'args self, qb: &mut QueryBuilder<'args, Postgres>) {
        qb.push("*");
    }
}

impl IntoSql for All {
    fn into_sql(self, qb: &mut QueryBuilder<'_, Postgres>) {
        qb.push("*");
    }
}

impl IntoSql for &All {
    fn into_sql(self, qb: &mut QueryBuilder<'_, Postgres>) {
        qb.push("*");
    }
}

#[allow(dead_code)]
pub fn count<E>(expr: E) -> Count<E> {
    Count { expr }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Count<E> {
    expr: E,
}

impl<'args, E> ToSql<'args> for Count<E>
where
    E: ToSql<'args>,
{
    fn to_sql(&'args self, qb: &mut QueryBuilder<'args, Postgres>) {
        qb.push("COUNT(");
        self.expr.to_sql(qb);
        qb.push(")");
    }
}

impl<E> IntoSql for Count<E>
where
    E: IntoSql,
{
    fn into_sql(self, qb: &mut QueryBuilder<'_, Postgres>) {
        qb.push("COUNT(");
        self.expr.into_sql(qb);
        qb.push(")");
    }
}
