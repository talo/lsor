use sqlx::{Postgres, QueryBuilder};

use crate::{
    ord::{OrderBy, ToOrderBy},
    sql::IntoSql,
};

pub fn from<T>(table_reference: T, alias: &'static str) -> FromClause<T> {
    FromClause {
        table_reference,
        alias,
    }
}

#[derive(Clone, Copy, Debug)]
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

impl<T> IntoSql for FromClause<T>
where
    T: IntoSql,
{
    fn into_sql(self, qb: &mut QueryBuilder<'_, Postgres>) {
        qb.push("(");
        self.table_reference.into_sql(qb);
        qb.push(") AS ");
        qb.push(self.alias);
    }
}

impl<T> ToOrderBy for FromClause<T>
where
    T: ToOrderBy,
{
    type By = T::By;

    fn to_order_by(&self) -> OrderBy<Self::By> {
        self.table_reference.to_order_by()
    }
}

#[derive(Clone, Copy, Debug)]
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

    pub fn order_by<E>(self, order_by: OrderBy<E>) -> Ordered<Self, E> {
        Ordered {
            selection: self,
            order_by,
        }
    }

    pub fn limit(self, limit: i32) -> Limited<Self> {
        Limited {
            selection: self,
            limit,
        }
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

impl<E, FromItems> ToOrderBy for Select<E, FromItems>
where
    FromItems: ToOrderBy,
{
    type By = FromItems::By;

    fn to_order_by(&self) -> OrderBy<Self::By> {
        self.from_items.to_order_by()
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Filtered<S, C> {
    pub selection: S,
    pub condition: C,
}

impl<S, C> Filtered<S, C> {
    pub fn order_by<E>(self, order_by: OrderBy<E>) -> Ordered<Self, E> {
        Ordered {
            selection: self,
            order_by,
        }
    }

    pub fn limit(self, limit: i32) -> Limited<Self> {
        Limited {
            selection: self,
            limit,
        }
    }
}

impl<S, C> IntoSql for Filtered<S, C>
where
    S: IntoSql,
    C: IntoSql,
{
    fn into_sql(self, qb: &mut QueryBuilder<'_, Postgres>) {
        self.selection.into_sql(qb);
        qb.push(" WHERE ");
        self.condition.into_sql(qb);
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Ordered<S, E> {
    pub selection: S,
    pub order_by: OrderBy<E>,
}

impl<S, E> Ordered<S, E> {
    pub fn limit(self, limit: i32) -> Limited<Self> {
        Limited {
            selection: self,
            limit,
        }
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
        self.order_by.expr.into_sql(qb);
        qb.push(" ");
        self.order_by.order.into_sql(qb);
    }
}

impl<S, E> ToOrderBy for Ordered<S, E>
where
    E: Clone,
{
    type By = E;

    fn to_order_by(&self) -> OrderBy<Self::By> {
        OrderBy {
            expr: self.order_by.expr.clone(),
            order: self.order_by.order,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Limited<S> {
    pub selection: S,
    pub limit: i32,
}

impl<S> IntoSql for Limited<S>
where
    S: IntoSql,
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

#[derive(Clone, Copy, Debug)]
pub struct All;

impl IntoSql for All {
    fn into_sql(self, qb: &mut QueryBuilder<'_, Postgres>) {
        qb.push("*");
    }
}
