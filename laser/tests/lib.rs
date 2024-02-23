use chrono::{DateTime, Utc};
use laser::{
    sql::ToSql,
    table::{table, Table},
    upsert::{upsert, upsert_into},
    Laser,
};
use sqlx::{QueryBuilder, Row as _, Type};
use uuid::Uuid;

#[derive(Clone, Laser)]
pub struct Metadata {
    #[laser(pk)]
    pub id: Uuid,

    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Clone, Type)]
pub enum AccountTier {
    Free,
    Pro,
    Startup,
    Enterprise,
}

#[derive(Clone, Laser)]
#[laser(table = "accounts")]
pub struct Account {
    #[laser(flatten)]
    pub metadata: Metadata,
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
        tier: AccountTier::Free,
    };

    {
        let mut qb = QueryBuilder::default();
        let stmt = upsert_into(Account::table(), &account);
        stmt.to_sql(&mut qb);
        assert_eq!(
            qb.sql(),
            "INSERT INTO accounts (id, created_at, updated_at, deleted_at, tier) VALUES ($1, $2, $3, $4, $5) ON CONFLICT (id) DO UPDATE SET (created_at, updated_at, deleted_at, tier) = ($6, $7, $8, $9)"
        );
    }

    {
        let mut qb = QueryBuilder::default();
        let stmt = upsert_into(Account::table(), account.clone());
        stmt.to_sql(&mut qb);
        assert_eq!(
            qb.sql(),
            "INSERT INTO accounts (id, created_at, updated_at, deleted_at, tier) VALUES ($1, $2, $3, $4, $5) ON CONFLICT (id) DO UPDATE SET (created_at, updated_at, deleted_at, tier) = ($6, $7, $8, $9)"
        );
    }

    {
        let mut qb = QueryBuilder::default();
        let stmt = upsert(&account);
        stmt.to_sql(&mut qb);
        assert_eq!(
            qb.sql(),
            "INSERT INTO accounts (id, created_at, updated_at, deleted_at, tier) VALUES ($1, $2, $3, $4, $5) ON CONFLICT (id) DO UPDATE SET (created_at, updated_at, deleted_at, tier) = ($6, $7, $8, $9)"
        );
    }

    {
        let mut qb = QueryBuilder::default();
        let stmt = upsert(account);
        stmt.to_sql(&mut qb);
        assert_eq!(
            qb.sql(),
            "INSERT INTO accounts (id, created_at, updated_at, deleted_at, tier) VALUES ($1, $2, $3, $4, $5) ON CONFLICT (id) DO UPDATE SET (created_at, updated_at, deleted_at, tier) = ($6, $7, $8, $9)"
        );
    }
}
