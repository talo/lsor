use chrono::{DateTime, Utc};
use lsor_core::{
    column::{col, ColumnName},
    driver::{Driver, PushPrql},
    filter::Filterable,
    row::{upsert_into, IsPk, Row},
    sort::{Order, Sorting},
    table::table,
};
use sqlx::{postgres::PgRow, FromRow, Type};
use uuid::Uuid;

pub struct Metadata {
    pub name: Option<String>,
    pub description: Option<String>,
    pub tags: Vec<String>,
}

pub enum MetadataFilter {
    Name(<Option<String> as Filterable>::Filter),
    Description(<Option<String> as Filterable>::Filter),
    Tags(<Vec<String> as Filterable>::Filter),
}

pub enum MetadataSort {
    Name(Order),
    Description(Order),
}

impl PushPrql for MetadataSort {
    fn push_to_driver(&self, driver: &mut Driver) {
        match self {
            Self::Name(_) => {
                col("name").push_to_driver(driver);
            }
            Self::Description(_) => {
                col("description").push_to_driver(driver);
            }
        }
    }
}

impl Sorting for MetadataSort {
    fn order(&self) -> Order {
        match self {
            Self::Name(order) => *order,
            Self::Description(order) => *order,
        }
    }

    fn flip(&self) -> impl Sorting {
        match self {
            Self::Name(order) => Self::Name(order.flip()),
            Self::Description(order) => Self::Description(order.flip()),
        }
    }

    fn push_to_driver_with_order(&self, driver: &mut Driver) {
        match self {
            Self::Name(order) => {
                order.push_to_driver(driver);
                col("name").push_to_driver(driver);
            }
            Self::Description(order) => {
                order.push_to_driver(driver);
                col("description").push_to_driver(driver);
            }
        }
    }
}

impl Row for Metadata {
    fn column_names() -> impl Iterator<Item = (ColumnName, IsPk)> {
        (Some((col(stringify!(name)), false)).into_iter())
            .chain(Some((col(stringify!(description)), false)).into_iter())
            .chain(Some((col(stringify!(tags)), false)).into_iter())
    }

    fn push_column_values(&self, driver: &mut Driver) {
        self.name.push_to_driver(driver);
        driver.push(", ");
        self.description.push_to_driver(driver);
        driver.push(", ");
        self.tags.push_to_driver(driver);
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
        (Some((col(stringify!(id)), true)).into_iter())
            .chain(Some((col(stringify!(created_at)), false)).into_iter())
            .chain(Some((col(stringify!(updated_at)), false)).into_iter())
            .chain(Some((col(stringify!(deleted_at)), false)).into_iter())
            .chain(Some((col(stringify!(tier)), false)).into_iter())
            .chain(Metadata::column_names())
    }

    fn push_column_values(&self, driver: &mut Driver) {
        self.id.push_to_driver(driver);
        driver.push(", ");
        self.created_at.push_to_driver(driver);
        driver.push(", ");
        self.updated_at.push_to_driver(driver);
        driver.push(", ");
        self.deleted_at.push_to_driver(driver);
        driver.push(", ");
        self.tier.push_to_driver(driver);
        driver.push(", ");
        self.metadata.push_column_values(driver);
    }
}

impl Row for &Account {
    fn column_names() -> impl Iterator<Item = (ColumnName, IsPk)> {
        Account::column_names()
    }

    fn push_column_values(&self, driver: &mut Driver) {
        (*self).push_column_values(driver);
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
    let mut driver = Driver::new();
    account.push_column_values(&mut driver);
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
    assert_eq!(driver.prql(), "INSERT INTO accounts (id, created_at, updated_at, deleted_at, tier, name, description, tags) VALUES ($1, $2, $3, $4, $5, $6, $7, $8) ON CONFLICT (id) DO UPDATE SET (created_at, updated_at, deleted_at, tier, name, description, tags) = ($2, $3, $4, $5, $6, $7, $8)");
}
