use base64::Engine;
use chrono::{DateTime, TimeZone, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{database::HasValueRef, Decode, Postgres, TypeInfo, ValueRef as _};
use uuid::Uuid;

use crate::literal::Literal;

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub enum Cursor {
    DateTime,
    String,
    Uuid,
}

impl Cursor {
    pub fn infer(column: <Postgres as HasValueRef<'_>>::ValueRef) -> sqlx::Result<String> {
        Ok(match column.type_info().as_ref().name() {
            "TIMESTAMP" | "TIMESTAMPTZ" => DateTimeCursor::encode(
                <DateTime<Utc> as Decode<'_, Postgres>>::decode(column)
                    .map_err(sqlx::Error::Decode)?,
            ),
            "TEXT" | "VARCHAR" => StringCursor::encode(
                <String as Decode<'_, Postgres>>::decode(column).map_err(sqlx::Error::Decode)?,
            ),
            "UUID" => UuidCursor::encode(
                <Uuid as Decode<'_, Postgres>>::decode(column).map_err(sqlx::Error::Decode)?,
            ),
            x => {
                return Err(sqlx::Error::Decode(
                    format!("invalid cursor type: {}", x).into(),
                ))
            }
        })
    }

    pub fn decode(&self, encoded: &str) -> Literal {
        match self {
            Self::DateTime => Literal::DateTime(DateTimeCursor::decode(encoded)),
            Self::String => Literal::String(StringCursor::decode(encoded)),
            Self::Uuid => Literal::Uuid(UuidCursor::decode(encoded)),
        }
    }
}

impl From<DateTimeCursor> for Cursor {
    fn from(_cursor: DateTimeCursor) -> Self {
        Self::DateTime
    }
}

impl From<StringCursor> for Cursor {
    fn from(_cursor: StringCursor) -> Self {
        Self::String
    }
}

impl From<UuidCursor> for Cursor {
    fn from(_cursor: UuidCursor) -> Self {
        Self::Uuid
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct DateTimeCursor;

impl DateTimeCursor {
    pub fn decode(encoded: &str) -> DateTime<Utc> {
        base64::engine::general_purpose::STANDARD
            .decode(encoded)
            .ok()
            .and_then(|buf| buf.as_slice().try_into().ok())
            .map(|buf| Utc.timestamp_nanos(i64::from_be_bytes(buf)))
            .unwrap_or_else(|| {
                tracing::warn!("invalid datetime cursor '{}'", encoded);
                Self::min()
            })
    }

    pub fn encode(decoded: DateTime<Utc>) -> String {
        base64::engine::general_purpose::STANDARD.encode(
            decoded
                .timestamp_nanos_opt()
                .expect("timestamp must be valid")
                .to_be_bytes(),
        )
    }

    pub fn min() -> DateTime<Utc> {
        Utc.timestamp_nanos(i64::MIN)
    }

    pub fn max() -> DateTime<Utc> {
        Utc.timestamp_nanos(i64::MAX)
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct StringCursor;

impl StringCursor {
    pub fn decode(encoded: &str) -> String {
        base64::engine::general_purpose::STANDARD
            .decode(encoded)
            .ok()
            .and_then(|buf| String::from_utf8(buf.as_slice().to_vec()).ok())
            .unwrap_or_else(|| {
                tracing::warn!("invalid string cursor '{}'", encoded);
                Self::min()
            })
    }

    pub fn encode(decoded: String) -> String {
        base64::engine::general_purpose::STANDARD.encode(decoded.as_bytes())
    }

    pub fn min() -> String {
        "".to_string()
    }

    #[allow(dead_code)]
    pub fn max() -> String {
        "~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~".to_string()
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct UuidCursor;

impl UuidCursor {
    pub fn decode(encoded: &str) -> Uuid {
        base64::engine::general_purpose::STANDARD
            .decode(encoded)
            .ok()
            .and_then(|buf| buf.as_slice().try_into().ok())
            .map(Uuid::from_bytes)
            .unwrap_or_else(|| {
                tracing::warn!("invalid uuid cursor '{}'", encoded);
                Self::min()
            })
    }

    pub fn encode(decoded: Uuid) -> String {
        base64::engine::general_purpose::STANDARD.encode(decoded.as_bytes())
    }

    pub fn min() -> Uuid {
        Uuid::from_bytes([0; 16])
    }

    pub fn max() -> Uuid {
        Uuid::from_bytes([255; 16])
    }
}
