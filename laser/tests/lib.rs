use async_graphql::{EmptyMutation, EmptySubscription, Enum, Object, Schema, SimpleObject};
use chrono::{DateTime, Utc};
use laser::{
    sql::{IntoSql, ToSql},
    table::Table,
    upsert::{upsert, upsert_into},
    Cursor, DateTimeCursor, DateTimeFilter, Filterable, Laser, Pagination, StringFilter,
};
use laser_proc_macro::Filterable;
use sqlx::{QueryBuilder, Row as _, Type};
use uuid::Uuid;

#[derive(Clone, Laser, SimpleObject)]
pub struct Metadata {
    #[laser(pk)]
    pub id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Clone, Copy, Debug, Enum, Eq, Filterable, PartialEq, Type)]
#[laser("=", "<>")]
pub enum AccountTier {
    Free,
    Pro,
    Startup,
    Enterprise,
}

#[derive(Clone, Laser, SimpleObject)]
#[laser(table = "accounts")]
pub struct Account {
    #[graphql(flatten)]
    #[laser(flatten)]
    pub metadata: Metadata,
    pub name: String,
    pub tier: AccountTier,
}

#[test]
fn test_upsert() {
    let account = Account {
        metadata: Metadata {
            id: Uuid::new_v4(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            deleted_at: None,
        },
        name: "test".to_string(),
        tier: AccountTier::Free,
    };

    {
        let mut qb = QueryBuilder::default();
        let stmt = upsert_into(Account::table(), &account);
        stmt.to_sql(&mut qb);
        assert_eq!(
            qb.sql(),
            "INSERT INTO accounts (id, created_at, updated_at, deleted_at, name, tier) VALUES ($1, $2, $3, $4, $5, $6) ON CONFLICT (id) DO UPDATE SET (created_at, updated_at, deleted_at, name, tier) = ($2, $3, $4, $5, $6)"
        );
    }

    {
        let mut qb = QueryBuilder::default();
        let stmt = upsert_into(Account::table(), account.clone());
        stmt.to_sql(&mut qb);
        assert_eq!(
            qb.sql(),
            "INSERT INTO accounts (id, created_at, updated_at, deleted_at, name, tier) VALUES ($1, $2, $3, $4, $5, $6) ON CONFLICT (id) DO UPDATE SET (created_at, updated_at, deleted_at, name, tier) = ($2, $3, $4, $5, $6)"
        );
    }

    {
        let mut qb = QueryBuilder::default();
        let stmt = upsert(&account);
        stmt.to_sql(&mut qb);
        assert_eq!(
            qb.sql(),
            "INSERT INTO accounts (id, created_at, updated_at, deleted_at, name, tier) VALUES ($1, $2, $3, $4, $5, $6) ON CONFLICT (id) DO UPDATE SET (created_at, updated_at, deleted_at, name, tier) = ($2, $3, $4, $5, $6)"
        );
    }

    {
        let mut qb = QueryBuilder::default();
        let stmt = upsert(account);
        stmt.to_sql(&mut qb);
        assert_eq!(
            qb.sql(),
            "INSERT INTO accounts (id, created_at, updated_at, deleted_at, name, tier) VALUES ($1, $2, $3, $4, $5, $6) ON CONFLICT (id) DO UPDATE SET (created_at, updated_at, deleted_at, name, tier) = ($2, $3, $4, $5, $6)"
        );
    }
}

#[test]
fn select_page() {
    let after = DateTimeCursor::encode(Utc::now());
    let before = DateTimeCursor::encode(Utc::now());
    let first = 10;
    let last = 5;

    let mut qb = QueryBuilder::new("");
    let subquery = Account::table()
        .select(laser::all())
        .filter_by(
            AccountFilter::any([
                AccountFilter::tier(AccountTierFilter::Eq(AccountTier::Free)).into(),
                AccountFilter::tier(AccountTierFilter::Eq(AccountTier::Pro)).into(),
            ])
            .with_metadata(MetadataFilter::created_at(DateTimeFilter::Eq(Utc::now())))
            .with_name(StringFilter::Eq("test".to_string())),
        )
        .order_by(laser::col("id"), laser::desc());
    laser::select_page_items(
        &subquery,
        Pagination {
            after,
            before,
            first,
            last,
            cursor: Cursor::DateTime,
        },
    )
    .into_sql(&mut qb);
    assert_eq!(qb.into_sql(), "SELECT *, id AS cursor FROM (SELECT * FROM (SELECT * FROM (SELECT * FROM accounts WHERE ((tier = $1) OR (tier = $2)) AND created_at = $3 AND name = $4 ORDER BY id DESC) AS page_items_inner WHERE ((id) > ($5)) AND ((id) < ($6)) ORDER BY id DESC LIMIT $7) AS page_items_outer ORDER BY id ASC LIMIT $8) AS page_items ORDER BY id DESC");

    let mut qb = QueryBuilder::new("");
    laser::select_page_info(
        &subquery,
        Cursor::DateTime,
        DateTimeCursor::encode(Utc::now()),
        DateTimeCursor::encode(Utc::now()),
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
