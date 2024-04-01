use async_graphql::{EmptyMutation, EmptySubscription, Enum, Object, Schema, SimpleObject};
use chrono::{DateTime, Utc};
use laser::{
    sql::IntoSql,
    table::Table,
    upsert::{upsert, upsert_into},
    DateTimeCursor, Filterable, JSONFilter, JSONObjectFilter, Laser, Order, Pagination, ToOrderBy,
};
use serde::{Deserialize, Serialize};
use sqlx::{QueryBuilder, Type};
use uuid::Uuid;

#[derive(Clone, Laser, SimpleObject)]
pub struct Metadata {
    #[laser(primary_key)]
    pub id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
    #[laser(skip_sort_by)]
    pub tags: Vec<String>,
}

#[derive(Clone, Copy, Debug, Enum, Eq, Filterable, PartialEq, Type)]
#[laser("=", "<>")]
pub enum AccountTier {
    Free,
    Pro,
    Startup,
    Enterprise,
}

#[derive(Clone, Deserialize, SimpleObject, Serialize)]
pub struct AccountConfig {
    pub x: String,
    pub y: String,
    pub z: String,
}

impl sqlx::Type<sqlx::Postgres> for AccountConfig {
    fn type_info() -> <sqlx::Postgres as sqlx::Database>::TypeInfo {
        sqlx::types::JsonValue::type_info()
    }
}

impl<'q> sqlx::Encode<'q, sqlx::Postgres> for AccountConfig {
    fn encode_by_ref(
        &self,
        buf: &mut <sqlx::Postgres as sqlx::database::HasArguments<'q>>::ArgumentBuffer,
    ) -> sqlx::encode::IsNull {
        serde_json::to_value(self)
            .expect("argument must be valid json")
            .encode_by_ref(buf)
    }
}

impl<'r> sqlx::Decode<'r, sqlx::Postgres> for AccountConfig {
    fn decode(
        value: <sqlx::Postgres as sqlx::database::HasValueRef<'r>>::ValueRef,
    ) -> Result<Self, sqlx::error::BoxDynError> {
        Ok(serde_json::from_value(sqlx::types::JsonValue::decode(
            value,
        )?)?)
    }
}

#[derive(Clone, Laser, SimpleObject)]
#[laser(table = "accounts")]
pub struct Account {
    #[graphql(flatten)]
    #[laser(flatten)]
    pub metadata: Metadata,
    pub name: String,
    #[graphql(skip)]
    #[laser(skip_sort_by)]
    pub tier: AccountTier,
    #[laser(json, skip_sort_by)]
    pub config: AccountConfig,
}

#[test]
fn test_upsert() {
    let account = Account {
        metadata: Metadata {
            id: Uuid::new_v4(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            deleted_at: None,
            tags: vec![],
        },
        name: "test".to_string(),
        tier: AccountTier::Free,
        config: AccountConfig {
            x: "x".to_string(),
            y: "y".to_string(),
            z: "z".to_string(),
        },
    };

    // // TODO: Get this passing.
    // {
    //     let mut qb = QueryBuilder::default();
    //     upsert_into(Account::table(), &account).into_sql(&mut qb);
    //     assert_eq!(
    //         qb.sql(),
    //         "INSERT INTO accounts (id, created_at, updated_at, deleted_at, tags, name, tier) VALUES ($1, $2, $3, $4, $5, $6, $7) ON CONFLICT (id) DO UPDATE SET (created_at, updated_at, deleted_at, tags, name, tier) = ($2, $3, $4, $5, $6, $7)"
    //     );
    // }

    {
        let mut qb = QueryBuilder::default();
        upsert_into(Account::table(), account.clone()).into_sql(&mut qb);
        assert_eq!(
            qb.sql(),
            "INSERT INTO accounts (id, created_at, updated_at, deleted_at, tags, name, tier) VALUES ($1, $2, $3, $4, $5, $6, $7) ON CONFLICT (id) DO UPDATE SET (created_at, updated_at, deleted_at, tags, name, tier) = ($2, $3, $4, $5, $6, $7)"
        );
    }

    // // TODO: Get this passing.
    // {
    //     let mut qb = QueryBuilder::default();
    //     upsert(&account).into_sql(&mut qb);
    //     assert_eq!(
    //         qb.sql(),
    //         "INSERT INTO accounts (id, created_at, updated_at, deleted_at, tags, name, tier) VALUES ($1, $2, $3, $4, $5, $6, $7) ON CONFLICT (id) DO UPDATE SET (created_at, updated_at, deleted_at, tags, name, tier) = ($2, $3, $4, $5, $6, $7)"
    //     );
    // }

    {
        let mut qb = QueryBuilder::default();
        upsert(account).into_sql(&mut qb);
        assert_eq!(
            qb.sql(),
            "INSERT INTO accounts (id, created_at, updated_at, deleted_at, tags, name, tier) VALUES ($1, $2, $3, $4, $5, $6, $7) ON CONFLICT (id) DO UPDATE SET (created_at, updated_at, deleted_at, tags, name, tier) = ($2, $3, $4, $5, $6, $7)"
        );
    }
}

#[test]
fn select_page() {
    let after = Some(DateTimeCursor::encode(&Utc::now()));
    let before = Some(DateTimeCursor::encode(&Utc::now()));
    let first = 10;
    let last = 5;

    let mut qb = QueryBuilder::new("");
    let sort_by = AccountSortBy::Metadata(MetadataSortBy::Id(Order::Desc));
    let subquery = Account::table()
        .select(laser::all())
        .filter_by(AccountFieldsFilter {
            config: Some(JSONFilter::Eq(JSONObjectFilter {
                key: "x".to_string(),
                value: "y".to_string(),
            })),
            ..Default::default()
        })
        .order_by(sort_by.to_order_by());
    laser::select_page_items(
        subquery.clone(),
        Pagination {
            cursor: sort_by.cursor(),
            after,
            before,
            first,
            last,
        },
    )
    .into_sql(&mut qb);
    assert_eq!(qb.into_sql(), "SELECT *, id AS cursor FROM (SELECT * FROM (SELECT * FROM (SELECT * FROM accounts WHERE ((tier = $1) OR (tier = $2)) AND created_at = $3 AND name = $4 ORDER BY id DESC) AS page_items_inner WHERE ((id) > ($5)) AND ((id) < ($6)) ORDER BY id DESC LIMIT $7) AS page_items_outer ORDER BY id ASC LIMIT $8) AS page_items ORDER BY id DESC");

    let mut qb = QueryBuilder::new("");
    laser::select_page_info(
        subquery,
        sort_by.cursor(),
        DateTimeCursor::encode(&Utc::now()),
        DateTimeCursor::encode(&Utc::now()),
    )
    .into_sql(&mut qb);
    assert_eq!(qb.into_sql(), "SELECT COUNT(*) AS total_count, COUNT(CASE WHEN (id) < ($1) THEN 1 END) > 0 AS has_prev_page, COUNT(CASE WHEN (id) > ($2) THEN 1 END) > 0 AS has_next_page FROM (SELECT * FROM accounts WHERE ((tier = $3) OR (tier = $4)) AND created_at = $5 AND name = $6 ORDER BY id DESC) AS page_info");
}

#[test]
fn sdl() {
    struct Query;

    #[Object]
    impl Query {
        async fn accounts(&self, _filter: AccountFilter) -> Vec<Account> {
            vec![]
        }
    }

    let schema = Schema::build(Query, EmptyMutation, EmptySubscription).finish();

    println!("{}", schema.sdl());
}
