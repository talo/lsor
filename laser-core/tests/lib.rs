use chrono::{DateTime, Utc};
use laser_core::filter::Filterable;
use sqlx::{postgres::PgRow, FromRow};
use uuid::Uuid;

pub struct Metadata {
    pub name: Option<String>,
    pub description: Option<String>,
    pub tags: Vec<String>,
}

pub struct MetadataFilter {
    pub name: <Option<String> as Filterable>::Filter,
    pub description: <Option<String> as Filterable>::Filter,
    pub tags: <Vec<String> as Filterable>::Filter,
}

impl<'r> FromRow<'r, PgRow> for Metadata {
    fn from_row(row: &'r PgRow) -> Result<Self, sqlx::Error> {
        use sqlx::Row;

        Ok(Self {
            name: row.try_get("name")?,
            description: row.try_get("description")?,
            tags: row.try_get("tags")?,
        })
    }
}

impl Filterable for Metadata {
    type Filter = MetadataFilter;
}

pub struct User {
    pub id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
    pub email: String,
    pub metadata: Metadata,
}

pub struct UserFilter {
    pub id: <Uuid as Filterable>::Filter,
    pub created_at: <DateTime<Utc> as Filterable>::Filter,
    pub updated_at: <DateTime<Utc> as Filterable>::Filter,
    pub deleted_at: <Option<DateTime<Utc>> as Filterable>::Filter,
    pub email: <String as Filterable>::Filter,
    pub metadata: <Metadata as Filterable>::Filter,
}

impl<'r> FromRow<'r, PgRow> for User {
    fn from_row(row: &'r PgRow) -> Result<Self, sqlx::Error> {
        use sqlx::Row;

        Ok(Self {
            id: row.try_get("id")?,
            created_at: row.try_get("created_at")?,
            updated_at: row.try_get("updated_at")?,
            deleted_at: row.try_get("deleted_at")?,
            email: row.try_get("email")?,
            metadata: Metadata::from_row(row)?,
        })
    }
}

impl Filterable for User {
    type Filter = UserFilter;
}
