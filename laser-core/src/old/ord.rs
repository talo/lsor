use async_graphql::Enum;
use serde::{Deserialize, Serialize};
use sqlx::{Postgres, QueryBuilder};

use crate::sql::IntoSql;

pub trait ToOrderBy {
    type By;

    fn to_order_by(&self) -> OrderBy<Self::By>;
}

#[derive(Clone, Copy, Debug, Deserialize, Enum, Eq, Hash, PartialEq, Serialize)]
pub enum Order {
    Asc,
    Desc,
}

impl Order {
    pub fn flip(self) -> Self {
        match self {
            Self::Asc => Self::Desc,
            Self::Desc => Self::Asc,
        }
    }
}

impl IntoSql for Order {
    fn into_sql(self, qb: &mut QueryBuilder<'_, Postgres>) {
        match self {
            Self::Asc => qb.push("ASC"),
            Self::Desc => qb.push("DESC"),
        };
    }
}

pub fn asc() -> Order {
    Order::Asc
}

pub fn desc() -> Order {
    Order::Desc
}

pub fn order_by<E>(expr: E, order: Order) -> OrderBy<E> {
    OrderBy { expr, order }
}

#[derive(Clone, Copy, Debug)]
pub struct OrderBy<E> {
    pub expr: E,
    pub order: Order,
}

impl<E> ToOrderBy for OrderBy<E>
where
    E: Clone,
{
    type By = E;

    fn to_order_by(&self) -> OrderBy<Self::By> {
        Self {
            expr: self.expr.clone(),
            order: self.order,
        }
    }
}

impl<E> IntoSql for OrderBy<E>
where
    E: IntoSql,
{
    fn into_sql(self, qb: &mut QueryBuilder<'_, Postgres>) {
        qb.push("ORDER BY ");
        self.expr.into_sql(qb);
        qb.push(" ");
        self.order.into_sql(qb);
    }
}
