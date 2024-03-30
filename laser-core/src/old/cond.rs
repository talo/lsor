use sqlx::{Postgres, QueryBuilder};

use crate::sql::IntoSql;

/// Create a conditional expression that checks for equality between two
/// expressions.
pub fn eq<LHS, RHS>(lhs: LHS, rhs: RHS) -> Eq<LHS, RHS> {
    Eq { lhs, rhs }
}

#[derive(Clone, Copy, Debug)]
pub struct Eq<LHS, RHS> {
    lhs: LHS,
    rhs: RHS,
}

impl<LHS, RHS> IntoSql for Eq<LHS, RHS>
where
    LHS: IntoSql,
    RHS: IntoSql,
{
    fn into_sql(self, qb: &mut QueryBuilder<'_, Postgres>) {
        qb.push("(");
        self.lhs.into_sql(qb);
        qb.push(") = (");
        self.rhs.into_sql(qb);
        qb.push(")");
    }
}

/// Create a conditional expression that checks for inequality between two
/// expressions.
pub fn neq<LHS, RHS>(lhs: LHS, rhs: RHS) -> Neq<LHS, RHS> {
    Neq { lhs, rhs }
}

#[derive(Clone, Copy, Debug)]
pub struct Neq<LHS, RHS> {
    lhs: LHS,
    rhs: RHS,
}

impl<LHS, RHS> IntoSql for Neq<LHS, RHS>
where
    LHS: IntoSql,
    RHS: IntoSql,
{
    fn into_sql(self, qb: &mut QueryBuilder<'_, Postgres>) {
        qb.push("(");
        self.lhs.into_sql(qb);
        qb.push(") <> (");
        self.rhs.into_sql(qb);
        qb.push(")");
    }
}

/// Create a conditional expression that checks for nullity of an expression.
pub fn is_null<E>(expr: E) -> IsNull<E> {
    IsNull { expr }
}

#[derive(Clone, Copy, Debug)]
pub struct IsNull<E> {
    expr: E,
}

impl<E> IntoSql for IsNull<E>
where
    E: IntoSql,
{
    fn into_sql(self, qb: &mut QueryBuilder<'_, Postgres>) {
        qb.push("(");
        self.expr.into_sql(qb);
        qb.push(") IS NULL");
    }
}

/// Create a conditional expression that checks for non-nullity of an
/// expression.
pub fn is_not_null<E>(expr: E) -> IsNotNull<E> {
    IsNotNull { expr }
}

#[derive(Clone, Copy, Debug)]
pub struct IsNotNull<E> {
    expr: E,
}

impl<E> IntoSql for IsNotNull<E>
where
    E: IntoSql,
{
    fn into_sql(self, qb: &mut QueryBuilder<'_, Postgres>) {
        qb.push("(");
        self.expr.into_sql(qb);
        qb.push(") IS NOT NULL");
    }
}

/// Create a conditional expression that checks for less-than inequality between
/// two expressions.
pub fn lt<LHS, RHS>(lhs: LHS, rhs: RHS) -> Lt<LHS, RHS> {
    Lt { lhs, rhs }
}

#[derive(Clone, Copy, Debug)]
pub struct Lt<LHS, RHS> {
    lhs: LHS,
    rhs: RHS,
}

impl<LHS, RHS> IntoSql for Lt<LHS, RHS>
where
    LHS: IntoSql,
    RHS: IntoSql,
{
    fn into_sql(self, qb: &mut QueryBuilder<'_, Postgres>) {
        qb.push("(");
        self.lhs.into_sql(qb);
        qb.push(") < (");
        self.rhs.into_sql(qb);
        qb.push(")");
    }
}

/// Create a conditional expression that checks for less-than-or-equal
/// inequality between two expressions.
pub fn lte<LHS, RHS>(lhs: LHS, rhs: RHS) -> Lte<LHS, RHS> {
    Lte { lhs, rhs }
}

#[derive(Clone, Copy, Debug)]
pub struct Lte<LHS, RHS> {
    lhs: LHS,
    rhs: RHS,
}

impl<LHS, RHS> IntoSql for Lte<LHS, RHS>
where
    LHS: IntoSql,
    RHS: IntoSql,
{
    fn into_sql(self, qb: &mut QueryBuilder<'_, Postgres>) {
        qb.push("(");
        self.lhs.into_sql(qb);
        qb.push(") <= (");
        self.rhs.into_sql(qb);
        qb.push(")");
    }
}

/// Create a conditional expression that checks for greater-than inequality
/// between two expressions.
pub fn gt<LHS, RHS>(lhs: LHS, rhs: RHS) -> Gt<LHS, RHS> {
    Gt { lhs, rhs }
}

#[derive(Clone, Copy, Debug)]
pub struct Gt<LHS, RHS> {
    lhs: LHS,
    rhs: RHS,
}

impl<LHS, RHS> IntoSql for Gt<LHS, RHS>
where
    LHS: IntoSql,
    RHS: IntoSql,
{
    fn into_sql(self, qb: &mut QueryBuilder<'_, Postgres>) {
        qb.push("(");
        self.lhs.into_sql(qb);
        qb.push(") > (");
        self.rhs.into_sql(qb);
        qb.push(")");
    }
}

/// Create a conditional expression that checks for greater-than-or-equal
/// inequality between two expressions.
pub fn gte<LHS, RHS>(lhs: LHS, rhs: RHS) -> Gte<LHS, RHS> {
    Gte { lhs, rhs }
}

#[derive(Clone, Copy, Debug)]
pub struct Gte<LHS, RHS> {
    lhs: LHS,
    rhs: RHS,
}

impl<LHS, RHS> IntoSql for Gte<LHS, RHS>
where
    LHS: IntoSql,
    RHS: IntoSql,
{
    fn into_sql(self, qb: &mut QueryBuilder<'_, Postgres>) {
        qb.push("(");
        self.lhs.into_sql(qb);
        qb.push(") >= (");
        self.rhs.into_sql(qb);
        qb.push(")");
    }
}

/// Create a conditional expression that checks for string likeness between two
/// string expressions.
pub fn like<LHS, RHS>(lhs: LHS, rhs: RHS) -> Like<LHS, RHS> {
    Like { lhs, rhs }
}

#[derive(Clone, Copy, Debug)]
pub struct Like<LHS, RHS> {
    lhs: LHS,
    rhs: RHS,
}

impl<LHS, RHS> IntoSql for Like<LHS, RHS>
where
    LHS: IntoSql,
    RHS: IntoSql,
{
    fn into_sql(self, qb: &mut QueryBuilder<'_, Postgres>) {
        qb.push("(");
        self.lhs.into_sql(qb);
        qb.push(") LIKE (");
        self.rhs.into_sql(qb);
        qb.push(")");
    }
}

/// Create a conditional expression that checks for inclusion in a list.
pub fn is_in<LHS, RHS>(lhs: LHS, rhs: RHS) -> IsIn<LHS, RHS> {
    IsIn { lhs, rhs }
}

#[derive(Clone, Copy, Debug)]
pub struct IsIn<LHS, RHS> {
    lhs: LHS,
    rhs: RHS,
}

impl<LHS, RHS> IntoSql for IsIn<LHS, RHS>
where
    LHS: IntoSql,
    RHS: IntoSql,
{
    fn into_sql(self, qb: &mut QueryBuilder<'_, Postgres>) {
        qb.push("(");
        self.lhs.into_sql(qb);
        qb.push(") IN (");
        self.rhs.into_sql(qb);
        qb.push(")");
    }
}

/// Create a conditional expression that checks for exclusion from a list.
pub fn is_not_in<LHS, RHS>(lhs: LHS, rhs: RHS) -> IsNotIn<LHS, RHS> {
    IsNotIn { lhs, rhs }
}

#[derive(Clone, Copy, Debug)]
pub struct IsNotIn<LHS, RHS> {
    lhs: LHS,
    rhs: RHS,
}

impl<LHS, RHS> IntoSql for IsNotIn<LHS, RHS>
where
    LHS: IntoSql,
    RHS: IntoSql,
{
    fn into_sql(self, qb: &mut QueryBuilder<'_, Postgres>) {
        qb.push("(");
        self.lhs.into_sql(qb);
        qb.push(") NOT IN (");
        self.rhs.into_sql(qb);
        qb.push(")");
    }
}

/// Create a conditional and expression on two conditions.
pub fn and<LHS, RHS>(lhs: LHS, rhs: RHS) -> And<LHS, RHS> {
    And { lhs, rhs }
}

#[derive(Clone, Copy, Debug)]
pub struct And<LHS, RHS> {
    lhs: LHS,
    rhs: RHS,
}

impl<LHS, RHS> IntoSql for And<LHS, RHS>
where
    LHS: IntoSql,
    RHS: IntoSql,
{
    fn into_sql(self, qb: &mut QueryBuilder<'_, Postgres>) {
        qb.push("(");
        self.lhs.into_sql(qb);
        qb.push(") AND (");
        self.rhs.into_sql(qb);
        qb.push(")");
    }
}

/// Create a conditional or expression on two conditions.
pub fn or<LHS, RHS>(lhs: LHS, rhs: RHS) -> Or<LHS, RHS> {
    Or { lhs, rhs }
}

#[derive(Clone, Copy, Debug)]
pub struct Or<LHS, RHS> {
    lhs: LHS,
    rhs: RHS,
}

impl<LHS, RHS> IntoSql for Or<LHS, RHS>
where
    LHS: IntoSql,
    RHS: IntoSql,
{
    fn into_sql(self, qb: &mut QueryBuilder<'_, Postgres>) {
        qb.push("(");
        self.lhs.into_sql(qb);
        qb.push(") OR (");
        self.rhs.into_sql(qb);
        qb.push(")");
    }
}

/// Create a conditional not expression on a condition.
pub fn not<E>(expr: E) -> Not<E> {
    Not { expr }
}

#[derive(Clone, Copy, Debug)]
pub struct Not<E> {
    expr: E,
}

impl<E> IntoSql for Not<E>
where
    E: IntoSql,
{
    fn into_sql(self, qb: &mut QueryBuilder<'_, Postgres>) {
        qb.push("NOT (");
        self.expr.into_sql(qb);
        qb.push(")");
    }
}

pub fn if_then_else<L, R>(cond: bool, left: L, right: R) -> Either<L, R> {
    if cond {
        Either::Left(left)
    } else {
        Either::Right(right)
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Either<L, R> {
    Left(L),
    Right(R),
}

impl<L, R> IntoSql for Either<L, R>
where
    L: IntoSql,
    R: IntoSql,
{
    fn into_sql(self, qb: &mut QueryBuilder<'_, Postgres>) {
        match self {
            Self::Left(left) => left.into_sql(qb),
            Self::Right(right) => right.into_sql(qb),
        }
    }
}
