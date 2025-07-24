use serde::{Deserialize, Serialize};
use sqlx::{
    decode::Decode,
    encode::{Encode, IsNull},
    error::BoxDynError,
    postgres::{PgHasArrayType, PgTypeInfo},
    types::Type,
    Database, Postgres, ValueRef,
};
use std::ops::{Deref, DerefMut};

#[derive(
    Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Serialize, Deserialize,
)]
#[serde(transparent)]
pub struct Wrapper<T: ?Sized>(pub T);

impl<T> From<T> for Wrapper<T> {
    fn from(value: T) -> Self {
        Self(value)
    }
}

impl<T> Deref for Wrapper<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for Wrapper<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T> AsRef<T> for Wrapper<T> {
    fn as_ref(&self) -> &T {
        &self.0
    }
}

impl<T> AsMut<T> for Wrapper<T> {
    fn as_mut(&mut self) -> &mut T {
        &mut self.0
    }
}

impl<T> Type<Postgres> for Wrapper<T> {
    fn type_info() -> <Postgres as Database>::TypeInfo {
        serde_json::Value::type_info()
    }
}

impl<T> PgHasArrayType for Wrapper<T> {
    fn array_type_info() -> PgTypeInfo {
        serde_json::Value::array_type_info()
    }
}

impl<'r, T> Decode<'r, Postgres> for Wrapper<T>
where
    T: for<'de> Deserialize<'de>,
{
    fn decode(sqlx_value: <Postgres as Database>::ValueRef<'r>) -> Result<Self, BoxDynError> {
        let json_value = if sqlx_value.is_null() {
            serde_json::Value::Null
        } else {
            serde_json::Value::decode(sqlx_value)?
        };
        Ok(Wrapper(serde_json::from_value::<T>(json_value)?))
    }
}

impl<'q, T> Encode<'q, Postgres> for Wrapper<T>
where
    T: Serialize + Clone,
{
    fn encode_by_ref(
        &self,
        buf: &mut <Postgres as Database>::ArgumentBuffer<'q>,
    ) -> Result<IsNull, BoxDynError> {
        let value = serde_json::to_value(self.0.clone())?;
        match &value {
            serde_json::Value::Null => Ok(IsNull::Yes),
            _ => value.encode(buf),
        }
    }
}
