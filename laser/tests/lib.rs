use chrono::{DateTime, Utc};
use laser::{
    column::{column, ColumnName, Columns},
    sql::ToSql,
    table::{table, Table, TableName},
    upsert::{upsert, upsert_into},
    value::ToValues,
};
use sqlx::{postgres::PgRow, FromRow, QueryBuilder, Row as _};
use uuid::Uuid;

#[derive(Clone)]
pub struct Metadata {
    pub id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

impl<'r> FromRow<'r, PgRow> for Metadata {
    fn from_row(row: &'r PgRow) -> sqlx::Result<Self> {
        Ok(Self {
            id: row.try_get("id")?,
            created_at: row.try_get("created_at")?,
            updated_at: row.try_get("updated_at")?,
            deleted_at: row.try_get("deleted_at")?,
        })
    }
}

impl Columns for Metadata {
    type D = &'static str;

    fn columns() -> impl Iterator<Item = (ColumnName<Self::D>, bool)> {
        [
            (column("id"), true),
            (column("created_at"), false),
            (column("updated_at"), false),
            (column("deleted_at"), false),
        ]
        .into_iter()
    }
}

impl ToValues for Metadata {
    fn to_values(&self) -> impl Iterator<Item = &dyn ToSql> {
        [
            &self.id as &dyn ToSql,
            &self.created_at as &dyn ToSql,
            &self.updated_at as &dyn ToSql,
            &self.deleted_at as &dyn ToSql,
        ]
        .into_iter()
    }
}

#[derive(Clone, sqlx::Type)]
pub enum AccountTier {
    Free,
    Pro,
    Startup,
    Enterprise,
}

#[derive(Clone)]
pub struct Account {
    pub metadata: Metadata,
    pub tier: AccountTier,
}

impl<'r> FromRow<'r, PgRow> for Account {
    fn from_row(row: &'r PgRow) -> sqlx::Result<Self> {
        Ok(Self {
            metadata: <_>::from_row(row)?,
            tier: row.try_get("tier")?,
        })
    }
}

impl Table for Account {
    type D = &'static str;

    fn table() -> TableName<Self::D> {
        table("accounts")
    }
}

impl Columns for Account {
    type D = &'static str;

    fn columns() -> impl Iterator<Item = (ColumnName<Self::D>, bool)> {
        Metadata::columns().chain(std::iter::once((column("tier"), false)))
    }
}

impl ToValues for Account {
    fn to_values(&self) -> impl Iterator<Item = &dyn ToSql> {
        self.metadata
            .to_values()
            .chain(std::iter::once(&self.tier as &dyn ToSql))
    }
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
