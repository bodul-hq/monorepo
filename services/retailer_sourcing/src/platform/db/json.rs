//! Typed JSONB metadata objects at the Diesel boundary.
use deserialize::Result;
use serde_json::from_slice;
use serde_json::to_vec;
use serialize::Result as SerializeResult;
use std::fmt::Debug as StdFmtDebug;

use diesel::deserialize::{self, FromSql};
use diesel::pg::{Pg, PgValue};
use diesel::serialize::{self, IsNull, Output, ToSql};
use diesel::sql_types::Jsonb;
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use std::io::Write;

#[derive(Debug, Clone, PartialEq, Eq, diesel::AsExpression, diesel::FromSqlRow)]
#[diesel(sql_type = Jsonb)]
pub struct Json<T>(pub T);

impl<T: DeserializeOwned> FromSql<Jsonb, Pg> for Json<T> {
    fn from_sql(value: PgValue<'_>) -> Result<Self> {
        let (version, body) = value.as_bytes().split_first().ok_or("Empty JSONB value")?;
        if *version != 1 {
            return Err("Unsupported JSONB encoding version".into());
        }
        // Serde also permits structs encoded as positional arrays; metadata
        // must use named fields in an object instead.
        if body.iter().find(|byte| !byte.is_ascii_whitespace()) != Some(&b'{') {
            return Err("Expected a JSONB metadata object".into());
        }
        Ok(Self(from_slice(body)?))
    }
}

impl<T: Serialize + StdFmtDebug> ToSql<Jsonb, Pg> for Json<T> {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Pg>) -> SerializeResult {
        let body = to_vec(&self.0)?;
        if body.first() != Some(&b'{') {
            return Err("Expected a JSONB metadata object".into());
        }
        out.write_all(&[1])?;
        out.write_all(&body)?;
        Ok(IsNull::No)
    }
}

/// Byte length of the stored artifact, shared by all pipeline stages.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Storage {
    pub content_size: u64,
}

#[cfg(test)]
mod tests {
    use serde_json::from_str;
    use serde_json::to_string;

    use super::Storage;

    #[test]
    fn storage_requires_an_unsigned_integer_and_known_fields() {
        for input in [
            "null",
            "[]",
            "{}",
            r#"{"content_size":null}"#,
            r#"{"content_size":"7"}"#,
            r#"{"content_size":1.5}"#,
            r#"{"content_size":-1}"#,
            r#"{"content_size":18446744073709551616}"#,
            r#"{"content_size":7,"body_size":7}"#,
        ] {
            assert!(from_str::<Storage>(input).is_err(), "{input}");
        }
        for content_size in [0, 7, u64::MAX] {
            let storage = Storage { content_size };
            let json = to_string(&storage).unwrap();
            assert_eq!(from_str::<Storage>(&json).unwrap(), storage);
        }
    }
}
