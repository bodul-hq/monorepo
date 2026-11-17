//! Strict persistence shape; legacy wire deserialization stays in retailer_sauce.

use deserialize::Result as DeserializeResult;
use diesel::AsExpression;
use diesel::FromSqlRow;
use diesel::{
    deserialize::{self, FromSql},
    pg::{Pg, PgValue},
    serialize::{self, Output, ToSql},
    sql_types::Jsonb,
};
use retailer_sauce::identifier::ExternalIdentifiers;
use serde_json::Value;
use serde_json::from_value as serde_json_from_value;
use serde_json::to_value;
use serialize::Result as SerializeResult;
use std::error::Error as ErrorError;

#[derive(Debug, Clone, PartialEq, AsExpression, FromSqlRow)]
#[diesel(sql_type = Jsonb)]
pub struct Identifier(pub ExternalIdentifiers);

impl Identifier {
    fn decode(value: Value) -> DeserializeResult<Self> {
        const KEYS: [&str; 3] = [
            "shopify_product_gid",
            "shopify_product_variation_gid",
            "shopify_handle",
        ];
        let object = value.as_object().ok_or("expected identifier object")?;
        if object.len() != KEYS.len() || !KEYS.iter().all(|key| object.contains_key(*key)) {
            return Err("expected exactly three Shopify identifier keys".into());
        }
        Ok(Self(serde_json_from_value(value)?))
    }

    fn encode(&self) -> Result<Value, Box<dyn ErrorError + Send + Sync>> {
        self.0.guard()?;
        Ok(to_value(&self.0)?)
    }
}

impl FromSql<Jsonb, Pg> for Identifier {
    fn from_sql(value: PgValue<'_>) -> DeserializeResult<Self> {
        Self::decode(<Value as FromSql<Jsonb, Pg>>::from_sql(value)?)
    }
}

impl ToSql<Jsonb, Pg> for Identifier {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Pg>) -> SerializeResult {
        let value = self.encode()?;
        <Value as ToSql<Jsonb, Pg>>::to_sql(&value, &mut out.reborrow())
    }
}

#[cfg(test)]
mod tests {
    use serde_json::from_value as decode_json_value;

    use super::*;
    use retailer_sauce::identifier::ShopifyHandle;
    use serde_json::json;

    #[test]
    fn exact_shape_and_values_round_trip() {
        for value in [
            json!({"shopify_product_gid":null,"shopify_product_variation_gid":null,"shopify_handle":null}),
            json!({"shopify_product_gid":"gid://shopify/Product/001","shopify_product_variation_gid":null,"shopify_handle":" ž \\\" 001 "}),
            json!({"shopify_product_gid":"01","shopify_product_variation_gid":"002","shopify_handle":"   "}),
        ] {
            let identifier = Identifier::decode(value.clone()).unwrap();
            assert_eq!(identifier.encode().unwrap(), value);
        }
    }

    #[test]
    fn rejects_invalid_shape_and_values_without_tightening_legacy_wire_defaults() {
        let full = json!({"shopify_product_gid":null,"shopify_product_variation_gid":null,"shopify_handle":null});
        for value in [
            Value::Null,
            json!([]),
            json!([null, null, null]),
            json!(1),
            json!("text"),
            json!({}),
        ] {
            assert!(Identifier::decode(value).is_err());
        }
        for key in [
            "shopify_product_gid",
            "shopify_product_variation_gid",
            "shopify_handle",
        ] {
            let mut missing = full.clone();
            missing.as_object_mut().unwrap().remove(key);
            assert!(Identifier::decode(missing).is_err());
            for invalid in [
                json!(0),
                json!(true),
                json!([]),
                json!({}),
                json!(""),
                json!("a\0b"),
            ] {
                let mut value = full.clone();
                value[key] = invalid;
                assert!(Identifier::decode(value).is_err());
            }
        }
        let mut extra = full;
        extra["gtin"] = json!("001");
        assert!(Identifier::decode(extra).is_err());
        assert_eq!(
            decode_json_value::<ExternalIdentifiers>(json!({})).unwrap(),
            ExternalIdentifiers::default()
        );
        for value in ["", "a\0b"] {
            let invalid = Identifier(ExternalIdentifiers {
                shopify_handle: Some(ShopifyHandle {
                    value: value.into(),
                }),
                ..Default::default()
            });
            assert!(invalid.encode().is_err());
        }
    }
}
