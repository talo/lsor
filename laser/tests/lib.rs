use chrono::{DateTime, Utc};
use laser::{
    column::col,
    driver::{Driver, PushPrql},
    filter::{DateTimeFilter, I32Filter, UuidFilter},
    row::upsert,
    sort::{DateTimeSort, I32Sort, Sorting, StringSort, UuidSort},
    Filter, Row, Sort, Type,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Copy, Debug, Eq, Filter, PartialEq, Row, Sort)]
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

#[derive(Clone, Debug, Deserialize, Eq, Filter, PartialEq, Row, Serialize, Sort)]
#[laser(json)]
pub struct AccountConfig {
    pub x: i32,
    pub y: String,
    pub z: Uuid,
}

#[derive(Clone, Debug, Eq, Filter, PartialEq, Row, Sort)]
#[laser(table = "accounts")]
pub struct Account {
    #[laser(pk)]
    pub id: Uuid,

    #[laser(skip_sort)]
    pub tier: AccountTier,

    pub config: AccountConfig,

    #[laser(flatten)]
    pub metadata: Metadata,
}

#[test]
fn test_enum_filter() {
    let mut driver = Driver::new();
    AccountTierFilter::Eq(AccountTier::Free).push_to_driver(&col("tier"), &mut driver);
    assert_eq!(driver.prql(), "tier == $1");
}

#[test]
fn test_struct_filter() {
    let mut driver = Driver::new();
    AccountFilter::Id(UuidFilter::Eq(Uuid::max())).push_to_driver(&mut driver);
    assert_eq!(driver.prql(), "accounts.id == $1");

    let mut driver = Driver::new();
    AccountFilter::All(vec![
        AccountFilter::Id(UuidFilter::Eq(Uuid::max())),
        AccountFilter::Any(vec![
            AccountFilter::Tier(AccountTierFilter::Eq(AccountTier::Free)),
            AccountFilter::Tier(AccountTierFilter::Eq(AccountTier::Pro)),
        ]),
    ])
    .push_to_driver(&mut driver);
    assert_eq!(
        driver.prql(),
        "(accounts.id == $1) && ((accounts.tier == $2) || (accounts.tier == $3))"
    );
}

#[test]
fn test_embedded_filter() {
    let mut driver = Driver::new();
    AccountFilter::Metadata(MetadataFilter::CreatedAt(DateTimeFilter::Eq(Utc::now())))
        .push_to_driver(&mut driver);
    assert_eq!(driver.prql(), "accounts.created_at == $1");
}

#[test]
fn test_json_filter() {
    let mut driver = Driver::new();
    AccountFilter::Config(AccountConfigFilter::X(I32Filter::Eq(1))).push_to_driver(&mut driver);
    assert_eq!(driver.prql(), "s\"accounts.config->'x'\" == $1");
}

#[test]
fn test_struct_sort() {
    use laser::driver::PushPrql;

    let mut driver = Driver::new();
    AccountSort::Id(UuidSort::Desc).push_to_driver(&mut driver);
    assert_eq!(driver.prql(), "id");

    let mut driver = Driver::new();
    AccountSort::Id(UuidSort::Desc).push_to_driver_with_order(&mut driver);
    assert_eq!(driver.prql(), "-id");
}

#[test]
fn test_embedded_sort() {
    let mut driver = Driver::new();
    AccountSort::Metadata(MetadataSort::CreatedAt(DateTimeSort::Desc)).push_to_driver(&mut driver);
    assert_eq!(driver.prql(), "created_at");

    let mut driver = Driver::new();
    AccountSort::Metadata(MetadataSort::CreatedAt(DateTimeSort::Desc))
        .push_to_driver_with_order(&mut driver);
    assert_eq!(driver.prql(), "-created_at");
}

#[test]
fn test_json_sort() {
    let mut driver = Driver::new();
    AccountSort::Config(AccountConfigSort::X(I32Sort::Desc)).push_to_driver(&mut driver);
    assert_eq!(driver.prql(), "s\"config->'x'\"");

    let mut driver = Driver::new();
    AccountSort::Config(AccountConfigSort::Y(StringSort::Desc))
        .push_to_driver_with_order(&mut driver);
    assert_eq!(driver.prql(), "-s\"config->'y'\"");
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
            z: Uuid::max(),
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
