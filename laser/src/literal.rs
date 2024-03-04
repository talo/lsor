use chrono::{DateTime, Utc};
use sqlx::{Postgres, QueryBuilder};
use uuid::Uuid;

use crate::sql::IntoSql;

/// A literal value.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Literal {
    Bool(bool),
    I32(i32),
    String(String),
    Uuid(Uuid),
    DateTime(DateTime<Utc>),
}

impl IntoSql for Literal {
    fn into_sql(self, qb: &mut QueryBuilder<'_, Postgres>) {
        match self {
            Self::Bool(x) => qb.push_bind(x),
            Self::I32(x) => qb.push_bind(x),
            Self::String(x) => qb.push_bind(x),
            Self::Uuid(x) => qb.push_bind(x),
            Self::DateTime(x) => qb.push_bind(x),
        };
    }
}
