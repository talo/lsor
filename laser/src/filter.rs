use async_graphql::OneofObject;
use chrono::{DateTime, Utc};
use sqlx::{Postgres, QueryBuilder};
use uuid::Uuid;

pub trait Filterable {
    type Filter;
}

impl Filterable for i32 {
    type Filter = I32Filter;
}

impl Filterable for Option<i32> {
    type Filter = I32Filter;
}

impl Filterable for String {
    type Filter = StringFilter;
}

impl Filterable for Option<String> {
    type Filter = StringFilter;
}

impl Filterable for Uuid {
    type Filter = UuidFilter;
}

impl Filterable for Option<Uuid> {
    type Filter = UuidFilter;
}

impl Filterable for DateTime<Utc> {
    type Filter = DateTimeFilter;
}

impl Filterable for Option<DateTime<Utc>> {
    type Filter = DateTimeFilter;
}

#[derive(Clone, Debug, OneofObject)]
#[graphql(rename_fields = "snake_case")]
pub enum I32Filter {
    Eq(i32),
    Ne(i32),
    Gt(i32),
    Ge(i32),
    Lt(i32),
    Le(i32),
}

impl I32Filter {
    pub fn into_sql(self, column_name: &'static str, qb: &mut QueryBuilder<'_, Postgres>) {
        match self {
            Self::Eq(x) => {
                qb.push(column_name);
                qb.push(" = ");
                qb.push_bind(x);
            }
            Self::Ne(x) => {
                qb.push(column_name);
                qb.push(" <> ");
                qb.push_bind(x);
            }
            Self::Gt(x) => {
                qb.push(column_name);
                qb.push(" > ");
                qb.push_bind(x);
            }
            Self::Ge(x) => {
                qb.push(column_name);
                qb.push(" >= ");
                qb.push_bind(x);
            }
            Self::Lt(x) => {
                qb.push(column_name);
                qb.push(" < ");
                qb.push_bind(x);
            }
            Self::Le(x) => {
                qb.push(column_name);
                qb.push(" <= ");
                qb.push_bind(x);
            }
        }
    }
}

#[derive(Clone, Debug, OneofObject)]
#[graphql(rename_fields = "snake_case")]
pub enum StringFilter {
    Eq(String),
    Ne(String),
    Gt(String),
    Ge(String),
    Lt(String),
    Le(String),
    Like(String),
    In(Vec<String>),
    NotIn(Vec<String>),
}

impl StringFilter {
    pub fn into_sql(&self, column_name: &'static str, qb: &mut QueryBuilder<'_, Postgres>) {
        match self {
            Self::Eq(x) => {
                qb.push(column_name);
                qb.push(" = ");
                qb.push_bind(x.clone());
            }
            Self::Ne(x) => {
                qb.push(column_name);
                qb.push(" <> ");
                qb.push_bind(x.clone());
            }
            Self::Gt(x) => {
                qb.push(column_name);
                qb.push(" > ");
                qb.push_bind(x.clone());
            }
            Self::Ge(x) => {
                qb.push(column_name);
                qb.push(" >= ");
                qb.push_bind(x.clone());
            }
            Self::Lt(x) => {
                qb.push(column_name);
                qb.push(" < ");
                qb.push_bind(x.clone());
            }
            Self::Le(x) => {
                qb.push(column_name);
                qb.push(" <= ");
                qb.push_bind(x.clone());
            }
            Self::Like(x) => {
                qb.push(column_name);
                qb.push(" LIKE ");
                qb.push_bind(x.clone());
            }
            Self::In(xs) => {
                qb.push(column_name);
                qb.push(" IN (");
                for (i, x) in xs.into_iter().enumerate() {
                    if i > 0 {
                        qb.push(", ");
                    }
                    qb.push_bind(x.clone());
                }
                qb.push(")");
            }
            Self::NotIn(xs) => {
                qb.push(column_name);
                qb.push(" NOT IN (");
                for (i, x) in xs.into_iter().enumerate() {
                    if i > 0 {
                        qb.push(", ");
                    }
                    qb.push_bind(x.clone());
                }
                qb.push(")");
            }
        }
    }
}

#[derive(Clone, Debug, OneofObject)]
#[graphql(rename_fields = "snake_case")]
pub enum UuidFilter {
    Eq(Uuid),
    Ne(Uuid),
    In(Vec<Uuid>),
}

impl UuidFilter {
    pub fn into_sql(&self, column_name: &'static str, qb: &mut QueryBuilder<'_, Postgres>) {
        match self {
            Self::Eq(x) => {
                qb.push(column_name);
                qb.push(" = ");
                qb.push_bind(*x);
            }
            Self::Ne(x) => {
                qb.push(column_name);
                qb.push(" <> ");
                qb.push_bind(*x);
            }
            Self::In(xs) => {
                qb.push(column_name);
                qb.push(" IN (");
                for (i, x) in xs.into_iter().enumerate() {
                    if i > 0 {
                        qb.push(", ");
                    }
                    qb.push_bind(*x);
                }
                qb.push(")");
            }
        }
    }
}

#[derive(Clone, Debug, OneofObject)]
#[graphql(rename_fields = "snake_case")]
pub enum DateTimeFilter {
    Eq(DateTime<Utc>),
    Ne(DateTime<Utc>),
    Gt(DateTime<Utc>),
    Ge(DateTime<Utc>),
    Lt(DateTime<Utc>),
    Le(DateTime<Utc>),
}

impl DateTimeFilter {
    pub fn into_sql(&self, column_name: &'static str, qb: &mut QueryBuilder<'_, Postgres>) {
        match self {
            Self::Eq(x) => {
                qb.push(column_name);
                qb.push(" = ");
                qb.push_bind(*x);
            }
            Self::Ne(x) => {
                qb.push(column_name);
                qb.push(" <> ");
                qb.push_bind(*x);
            }
            Self::Gt(x) => {
                qb.push(column_name);
                qb.push(" > ");
                qb.push_bind(*x);
            }
            Self::Ge(x) => {
                qb.push(column_name);
                qb.push(" >= ");
                qb.push_bind(*x);
            }
            Self::Lt(x) => {
                qb.push(column_name);
                qb.push(" < ");
                qb.push_bind(*x);
            }
            Self::Le(x) => {
                qb.push(column_name);
                qb.push(" <= ");
                qb.push_bind(*x);
            }
        }
    }
}