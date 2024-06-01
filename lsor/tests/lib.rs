use async_graphql::Enum;
use chrono::{DateTime, Utc};
use lsor::{
    column::col,
    driver::{Driver, PushPrql},
    filter::{DateTimeFilter, I32Filter, UuidFilter},
    row::upsert,
    sort::{DateTimeSort, I32Sort, Sorting, StringSort, UuidSort},
    Filter, Row, Sort, Type,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Copy, Debug, Eq, Filter, PartialEq, Row, Sort, Serialize, Deserialize)]
pub struct Metadata {
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Clone, Copy, Debug, Eq, Filter, PartialEq, Enum, Type, Serialize, Deserialize)]
#[lsor("==", "!=")]
pub enum AccountTier {
    Free,
    Pro,
    Startup,
    Enterprise,
}

#[derive(Clone, Debug, Deserialize, Eq, Filter, PartialEq, Row, Serialize, Sort)]
#[lsor(json)]
pub struct AccountConfig {
    pub x: i32,
    pub y: String,
    pub z: Uuid,
}

#[derive(Clone, Debug, Eq, Filter, PartialEq, Row, Sort)]
#[lsor(table = "accounts")]
pub struct Account {
    #[lsor(pk)]
    pub id: Uuid,

    #[lsor(skip_sort)]
    pub tier: AccountTier,

    #[lsor(skip_sort)]
    #[lsor(skip_filter)]
    pub tiers: Vec<AccountTier>,

    pub config: AccountConfig,

    #[lsor(flatten)]
    pub metadata: Metadata,
}

#[derive(Clone, Debug, Eq, Filter, PartialEq, Row, Sort, Serialize, Deserialize)]
#[lsor(json, table = "accounts")]
pub struct JsonAccount {
    #[lsor(pk)]
    pub id: Uuid,

    #[lsor(skip_sort)]
    pub tier: AccountTier,

    #[lsor(skip_sort)]
    #[lsor(skip_filter)]
    pub tiers: Vec<AccountTier>,

    pub config: AccountConfig,

    #[lsor(skip_sort)]
    #[lsor(skip_filter)]
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
    PushPrql::push_to_driver(&AccountFilter::Id(UuidFilter::Eq(Uuid::max())), &mut driver);
    assert_eq!(driver.prql(), "accounts.id == $1");

    let mut driver = Driver::new();
    PushPrql::push_to_driver(
        &AccountFilter::All(vec![
            AccountFilter::Id(UuidFilter::Eq(Uuid::max())),
            AccountFilter::Any(vec![
                AccountFilter::Tier(AccountTierFilter::Eq(AccountTier::Free)),
                AccountFilter::Tier(AccountTierFilter::Eq(AccountTier::Pro)),
            ]),
        ]),
        &mut driver,
    );
    assert_eq!(
        driver.prql(),
        "(accounts.id == $1) && ((accounts.tier == $2) || (accounts.tier == $3))"
    );
}

#[test]
fn test_embedded_filter() {
    let mut driver = Driver::new();
    PushPrql::push_to_driver(
        &AccountFilter::Metadata(MetadataFilter::CreatedAt(DateTimeFilter::Eq(Utc::now()))),
        &mut driver,
    );
    assert_eq!(driver.prql(), "accounts.created_at == $1");
}

#[test]
fn test_json_filter() {
    use lsor::driver::PushPrql;
    let mut driver = Driver::new();
    PushPrql::push_to_driver(
        &AccountFilter::Config(AccountConfigFilter::X(I32Filter::Eq(1))),
        &mut driver,
    );
    assert_eq!(driver.prql(), "s\"accounts.config->'x'\" == $1");

    let mut driver = Driver::new();
    JsonAccountFilter::Tier(AccountTierFilter::Eq(AccountTier::Free))
        .push_to_driver(&lsor::col("account"), &mut driver);
    assert_eq!(driver.prql(), "s\"accounts.config->'x'\" == $1");
}

#[test]
fn test_struct_sort() {
    use lsor::driver::PushPrql;

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
        tiers: vec![AccountTier::Pro],
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
    assert_eq!(
        driver.prql(),
        "INSERT INTO accounts (id, tier, tiers, config, created_at, updated_at, deleted_at) VALUES ($1, $2, $3, $4, $5, $6, $7) ON CONFLICT (id) DO UPDATE SET (tier, tiers, config, created_at, updated_at, deleted_at) = ($2, $3, $4, $5, $6, $7)"
    );
}
