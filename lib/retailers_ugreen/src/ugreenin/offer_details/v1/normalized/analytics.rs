//! Checks Liquid's record against Shopify's analytics `meta`, which Shopify renders itself
//! rather than an app. Ids, handles, prices and SKUs agree on all 1,408 saved pages, so any
//! disagreement means one of the two was misread. Nothing here crosses into the normalized
//! document.

use super::super::raw::{analytics, productjson};
use super::{NormalizeError, non_empty};

pub(super) fn check(
    analytics: &analytics::Product,
    record: &productjson::ProductJson,
) -> Result<(), NormalizeError> {
    if analytics.id != record.id {
        return Err(NormalizeError::AnalyticsProductMismatch {
            field: "id".to_string(),
            analytics: analytics.id.to_string(),
            record: record.id.to_string(),
        });
    }
    if analytics.handle != record.handle {
        return Err(NormalizeError::AnalyticsProductMismatch {
            field: "handle".to_string(),
            analytics: analytics.handle.clone(),
            record: record.handle.clone(),
        });
    }

    // Both list the variants in the shop's order.
    let analytics_ids: Vec<u64> = analytics
        .variants
        .iter()
        .map(|variant| variant.id)
        .collect();
    let record_ids: Vec<u64> = record.variants.iter().map(|variant| variant.id).collect();
    if analytics_ids != record_ids {
        return Err(NormalizeError::AnalyticsVariantIdsMismatch {
            analytics: analytics_ids,
            record: record_ids,
        });
    }

    for (seen, variant) in analytics.variants.iter().zip(&record.variants) {
        let mismatch = |field: &str, analytics: String, record: String| {
            Err(NormalizeError::AnalyticsVariantMismatch {
                variant_id: variant.id,
                field: field.to_string(),
                analytics,
                record,
            })
        };

        if seen.price != variant.price {
            return mismatch("price", seen.price.to_string(), variant.price.to_string());
        }
        let record_sku = variant.sku.clone().and_then(non_empty);
        if non_empty(seen.sku.clone()) != record_sku {
            return mismatch("sku", seen.sku.clone(), format!("{record_sku:?}"));
        }
    }

    Ok(())
}
