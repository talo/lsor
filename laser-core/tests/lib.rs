use chrono::{DateTime, Utc};
use laser_core::{
    column::{col, ColumnName},
    driver::{Driver, PushPrql},
    filter::Filterable,
    row::{upsert_into, IsPk, Row},
    table::table,
};
use sqlx::{postgres::PgRow, FromRow, Type};
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

impl Row for Metadata {
    fn column_names() -> impl Iterator<Item = (ColumnName, IsPk)> {
        (Some((col(stringify!(name)), false)).into_iter())
            .chain(Some((col(stringify!(description)), false)).into_iter())
            .chain(Some((col(stringify!(tags)), false)).into_iter())
    }

    fn column_values(&self) -> impl Iterator<Item = (&dyn PushPrql, bool)> {
        (Some((&self.name as &_, false)).into_iter())
            .chain(Some((&self.description as &_, false)).into_iter())
            .chain(Some((&self.tags as &_, false)).into_iter())
    }
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

#[derive(Type)]
pub enum AccountTier {
    Free,
    Pro,
    Startup,
    Enterprise,
}

impl PushPrql for AccountTier {
    fn push_to_driver(&self, driver: &mut Driver) {
        driver.push_bind(self);
    }
}

pub enum AccountTierFilter {
    Eq(AccountTier),
    Ne(AccountTier),
}

impl Filterable for AccountTier {
    type Filter = AccountTierFilter;
}

pub struct Account {
    pub id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
    pub tier: AccountTier,
    pub metadata: Metadata,
}

pub struct AccountFilter {
    pub id: <Uuid as Filterable>::Filter,
    pub created_at: <DateTime<Utc> as Filterable>::Filter,
    pub updated_at: <DateTime<Utc> as Filterable>::Filter,
    pub deleted_at: <Option<DateTime<Utc>> as Filterable>::Filter,
    pub tier: <AccountTier as Filterable>::Filter,
    pub metadata: <Metadata as Filterable>::Filter,
}

impl Row for Account {
    fn column_names() -> impl Iterator<Item = (ColumnName, IsPk)> {
        (Some((col(stringify!(id)), false)).into_iter())
            .chain(Some((col(stringify!(created_at)), false)).into_iter())
            .chain(Some((col(stringify!(updated_at)), false)).into_iter())
            .chain(Some((col(stringify!(deleted_at)), false)).into_iter())
            .chain(Some((col(stringify!(tier)), false)).into_iter())
            .chain(Metadata::column_names())
    }

    fn column_values(&self) -> impl Iterator<Item = (&dyn PushPrql, IsPk)> {
        (Some((&self.id as &_, true)).into_iter())
            .chain(Some((&self.created_at as &_, false)).into_iter())
            .chain(Some((&self.updated_at as &_, false)).into_iter())
            .chain(Some((&self.deleted_at as &_, false)).into_iter())
            .chain(Some((&self.tier as &_, false)).into_iter())
            .chain(self.metadata.column_values())
    }
}

impl Row for &Account {
    fn column_names() -> impl Iterator<Item = (ColumnName, IsPk)> {
        Account::column_names()
    }

    fn column_values(&self) -> impl Iterator<Item = (&dyn PushPrql, IsPk)> {
        [
            (&self.id as &dyn PushPrql, true),
            (&self.created_at as &dyn PushPrql, false),
            (&self.updated_at as &dyn PushPrql, false),
            (&self.deleted_at as &dyn PushPrql, false),
            (&self.tier as &dyn PushPrql, false),
        ]
        .into_iter()
        .chain(self.metadata.column_values())
    }
}

impl<'r> FromRow<'r, PgRow> for Account {
    fn from_row(row: &'r PgRow) -> Result<Self, sqlx::Error> {
        use sqlx::Row;

        Ok(Self {
            id: row.try_get("id")?,
            created_at: row.try_get("created_at")?,
            updated_at: row.try_get("updated_at")?,
            deleted_at: row.try_get("deleted_at")?,
            tier: row.try_get("tier")?,
            metadata: Metadata::from_row(row)?,
        })
    }
}

impl Filterable for Account {
    type Filter = AccountFilter;
}

#[test]
fn test_column_names() {
    let column_names = Account::column_names().collect::<Vec<_>>();
    assert_eq!(
        column_names,
        vec![
            (col("id"), true),
            (col("created_at"), false),
            (col("updated_at"), false),
            (col("deleted_at"), false),
            (col("tier"), false),
            (col("name"), false),
            (col("description"), false),
            (col("tags"), false),
        ]
    );
}

#[test]
fn test_column_values() {
    let account = Account {
        id: Uuid::new_v4(),
        created_at: Utc::now(),
        updated_at: Utc::now(),
        deleted_at: None,
        tier: AccountTier::Free,
        metadata: Metadata {
            name: Some("name".to_string()),
            description: Some("description".to_string()),
            tags: vec!["tag".to_string()],
        },
    };
    let column_values = account.column_values().collect::<Vec<_>>();
    let mut driver = Driver::new();
    for (i, (push_prql, _is_pk)) in column_values.into_iter().enumerate() {
        if i > 0 {
            driver.push(", ");
        }
        push_prql.push_to_driver(&mut driver);
    }
    assert_eq!(driver.prql(), "$1, $2, $3, $4, $5, $6, $7, $8");
}

#[test]
fn test_upsert() {
    let account = Account {
        id: Uuid::new_v4(),
        created_at: Utc::now(),
        updated_at: Utc::now(),
        deleted_at: None,
        tier: AccountTier::Free,
        metadata: Metadata {
            name: Some("name".to_string()),
            description: Some("description".to_string()),
            tags: vec!["tag".to_string()],
        },
    };
    let mut driver = Driver::new();
    upsert_into(table("accounts"), &account).push_to_driver(&mut driver);
    assert_eq!(driver.prql(), "s'INSERT INTO accounts (id, created_at, updated_at, deleted_at, tier, name, description, tags) VALUES ($1, $2, $3, $4, $5, $6, $7, $8) ON CONFLICT (id) DO UPDATE SET (created_at, updated_at, deleted_at, tier, name, description, tags) = ($2, $3, $4, $5, $6, $7, $8)'");
}
