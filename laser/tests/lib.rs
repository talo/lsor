use chrono::{DateTime, Utc};
use laser::{
    driver::{Driver, PushPrql},
    filter::{DateTimeFilter, UuidFilter},
    row::upsert,
    Filter, Row, Type,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Copy, Debug, Eq, Filter, PartialEq, Row)]
pub struct Metadata {
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Clone, Copy, Debug, Eq, Filter, PartialEq, async_graphql::Enum, Type)]
#[laser("==", "!=")]
pub enum AccountTier {
    Free,
    Pro,
    Startup,
    Enterprise,
}

#[derive(Clone, Debug, Deserialize, Eq, Filter, PartialEq, Row, Serialize)]
#[laser(json)]
pub struct AccountConfig {
    pub x: i32,
    pub y: String,
    pub z: bool,
}

#[derive(Clone, Debug, Eq, Filter, PartialEq, Row)]
#[laser(table = "accounts")]
pub struct Account {
    #[laser(pk)]
    pub id: Uuid,
    pub tier: AccountTier,

    pub config: AccountConfig,

    #[laser(flatten)]
    pub metadata: Metadata,
}

#[test]
fn test_enum_filter() {
    let mut driver = Driver::new();
    AccountTierFilter::Eq(AccountTier::Free).push_to_driver(&mut driver);
    assert_eq!(driver.prql(), " == $1");
}

#[test]
fn test_struct_filter() {
    let mut driver = Driver::new();
    AccountFilter::Id(UuidFilter::Eq(Uuid::max())).push_to_driver(&mut driver);
    assert_eq!(driver.prql(), "id == $1");
}

#[test]
fn test_embedded_filter() {
    let mut driver = Driver::new();
    AccountFilter::Metadata(MetadataFilter::CreatedAt(DateTimeFilter::Eq(Utc::now())))
        .push_to_driver(&mut driver);
    assert_eq!(driver.prql(), "created_at == $1");
}

#[test]
fn test_upsert() {
    let mut driver = Driver::new();
    upsert(Account {
        id: Uuid::new_v4(),
        tier: AccountTier::Free,
        config: AccountConfig {
            x: 1,
            y: "hello".to_string(),
            z: true,
        },
        metadata: Metadata {
            created_at: Utc::now(),
            updated_at: Utc::now(),
            deleted_at: None,
        },
    })
    .push_to_driver(&mut driver);
    assert_eq!(driver.prql(), "INSERT INTO accounts (id, tier, config, created_at, updated_at, deleted_at) VALUES ($1, $2, $3, $4, $5, $6) ON CONFLICT (id) DO UPDATE SET (tier, config, created_at, updated_at, deleted_at) = ($2, $3, $4, $5, $6)");
}
