use chrono::{DateTime, Utc};
use sqlx::{Postgres, QueryBuilder};
use uuid::Uuid;

use crate::sql::{IntoSql, ToSql};

/// A literal value.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Literal {
    I32(i32),
    String(String),
    Uuid(Uuid),
    DateTime(DateTime<Utc>),
}

impl<'args> ToSql<'args> for Literal {
    fn to_sql(&'args self, qb: &mut QueryBuilder<'args, Postgres>) {
        match self {
            Self::I32(x) => qb.push_bind(x),
            Self::String(x) => qb.push_bind(x),
            Self::Uuid(x) => qb.push_bind(x),
            Self::DateTime(x) => qb.push_bind(x),
        };
    }
}

impl IntoSql for Literal {
    fn into_sql(self, qb: &mut QueryBuilder<'_, Postgres>) {
        match self {
            Self::I32(x) => qb.push_bind(x),
            Self::String(x) => qb.push_bind(x),
            Self::Uuid(x) => qb.push_bind(x),
            Self::DateTime(x) => qb.push_bind(x),
        };
    }
}
