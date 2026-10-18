//! Turns normalized data into the product variations the rest of the system consumes: one
//! entry per variation, carrying its title, identifiers, images, price and URL.
//!
//! The storefront renders the offer twice - as a schema.org island and as its own product record -
//! and normalization keeps both. This is where they become one variation per variant, each
//! field taken from whichever rendering states it cleanly.

use super::normalized::{self, AnkerItalyComNormalizedDocument};
use crate::shared::{
    Gtin, IdentifierError, ShopifyHandle, ShopifyProductGid, ShopifyProductVariationGid, Sku,
};
use money::Money;
use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Error)]
pub enum StructuralizeError {
    #[error(transparent)]
    Identifier(#[from] IdentifierError),
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AnkerItalyComDocument {
    pub variations: Vec<ProductVariation>,
}

impl AnkerItalyComDocument {
    pub fn from_normalized(
        input: AnkerItalyComNormalizedDocument,
    ) -> Result<AnkerItalyComDocument, StructuralizeError> {
        let AnkerItalyComNormalizedDocument {
            ld_json: _,
            canonical_url,
            product,
        } = input;
        let variations_count = product.variants.len();

        let variations = product
            .variants
            .iter()
            .map(|variant| {
                let identifiers = ProductVariationIdentifier::from_normalized(&product, variant)?;
                let url = variation_url(
                    &canonical_url,
                    &identifiers.shopify_product_variation_gid,
                    variations_count,
                )?;

                Ok(ProductVariation {
                    title: product.title.clone(),
                    identifiers,
                    images: gallery(&product),
                    price: variant.price.clone(),
                    url,
                })
            })
            .collect::<Result<_, StructuralizeError>>()?;

        Ok(AnkerItalyComDocument { variations })
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ProductVariation {
    pub identifiers: ProductVariationIdentifier,
    /// The product's title, shared by every variation.
    pub title: String,
    /// The variation's own price, before any discount code.
    pub price: Money,
    pub url: String,
    /// The product record's gallery.
    pub images: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ProductVariationIdentifier {
    pub gtin: Option<Gtin>,
    pub sku: Sku,
    pub shopify_product_gid: ShopifyProductGid,
    pub shopify_product_variation_gid: ShopifyProductVariationGid,
    pub shopify_handle: ShopifyHandle,
}

impl ProductVariationIdentifier {
    fn from_normalized(
        product: &normalized::product::ProductRecord,
        variant: &normalized::product::ProductVariant,
    ) -> Result<ProductVariationIdentifier, StructuralizeError> {
        Ok(ProductVariationIdentifier {
            gtin: variant.barcode.clone().map(Gtin::try_from).transpose()?,
            sku: variant.sku.clone().try_into()?,
            shopify_product_gid: product.id.clone().try_into()?,
            shopify_product_variation_gid: variant.id.clone().try_into()?,
            shopify_handle: product.handle.clone().try_into()?,
        })
    }
}

// The island's `url` is the same canonical on every page; the page-level one is read so the
// storefront's own statement of its URL is the source.
fn variation_url(
    canonical_url: &str,
    gid: &ShopifyProductVariationGid,
    variations: usize,
) -> Result<String, StructuralizeError> {
    if variations < 2 {
        return Ok(canonical_url.to_owned());
    }

    Ok(format!("{}?variant={}", canonical_url, gid.numeric_id()?))
}

fn gallery(product: &normalized::product::ProductRecord) -> Vec<String> {
    product
        .images
        .iter()
        .map(|image| image.url.clone())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{AnkerItalyComDocument, StructuralizeError};
    use crate::ankeritalycom::offer_details::v1::normalized::AnkerItalyComNormalizedDocument;
    use crate::shared::IdentifierError;
    use html_extractor::serde_json::{self, Value, json};

    fn eur(amount: i64) -> Value {
        json!({"amount": amount, "currency": "EUR"})
    }

    fn variant(id: u64, sku: &str, selected_name: &str, colour: &str) -> Value {
        json!({
            "id": format!("gid://shopify/ProductVariant/{id}"), "title": colour, "sku": sku,
            "barcode": "194644248444", "available_for_sale": true,
            "currently_not_in_stock": false, "quantity_available": 3893,
            "price": eur(4999), "compare_at_price": null,
            "image": {"url": format!("https://cdn.shopify.com/{sku}.png")},
            "selected_options": [{"name": selected_name, "value": colour}],
            "discounts": []
        })
    }

    fn normalized_document() -> Value {
        json!({
            "ld_json": {"product": {"description": "CTA", "brand": "ankernordics"}},
            "canonical_url": "https://www.ankeritaly.com/products/a121c",
            "product": {
                "id": "gid://shopify/Product/1", "handle": "a121c",
                "title": "Anker Zolo Caricatore", "description": "Descrizione",
                "vendor": "Soundcore", "product_type": "", "updated_at": "2026-01-01T00:00:00Z",
                "available_for_sale": true, "price": eur(4999), "compare_at_price": null,
                "featured_image": {"url": "https://cdn.shopify.com/one.png"},
                "images": [{"url": "https://cdn.shopify.com/one.png"},
                           {"url": "https://cdn.shopify.com/two.png"}],
                "seo": {"title": null, "description": null},
                "options": [{"id": "gid://shopify/ProductOption/10843221262473", "name": "color",
                             "values": [{"label": "Nero", "hex_colors": ["#000000"]},
                                        {"label": "Bianco", "hex_colors": null}]}],
                "variants": [variant(1, "A121C311", "Color", "Nero"),
                             variant(2, "A121C321", "color", "Bianco")]
            }
        })
    }

    fn structuralized(input: &Value) -> Value {
        let result = crate::ankeritalycom::offer_details::v1::structuralize(&input.to_string())
            .expect("the document structuralizes");

        serde_json::from_str(&result.document_json.unwrap()).unwrap()
    }

    fn error(input: Value) -> StructuralizeError {
        let normalized: AnkerItalyComNormalizedDocument = serde_json::from_value(input).unwrap();

        AnkerItalyComDocument::from_normalized(normalized).unwrap_err()
    }

    #[test]
    fn a_variation_carries_its_title_identifiers_images_price_and_url_only() {
        let mut input = normalized_document();
        input["product"]["variants"]
            .as_array_mut()
            .unwrap()
            .truncate(1);

        assert_eq!(
            structuralized(&input),
            json!({"variations": [{
                "title": "Anker Zolo Caricatore",
                "identifiers": {
                    "gtin": "194644248444",
                    "sku": "A121C311",
                    "shopify_product_gid": "gid://shopify/Product/1",
                    "shopify_product_variation_gid": "gid://shopify/ProductVariant/1",
                    "shopify_handle": "a121c",
                },
                "images": ["https://cdn.shopify.com/one.png", "https://cdn.shopify.com/two.png"],
                "price": eur(4999),
                "url": "https://www.ankeritaly.com/products/a121c",
            }]})
        );
    }

    #[test]
    fn each_variation_of_a_multi_variant_document_gets_its_own_url() {
        let output = structuralized(&normalized_document());

        assert_eq!(
            output["variations"][0]["url"],
            "https://www.ankeritaly.com/products/a121c?variant=1"
        );
        assert_eq!(
            output["variations"][1]["url"],
            "https://www.ankeritaly.com/products/a121c?variant=2"
        );
    }

    #[test]
    fn a_single_variant_document_gets_the_bare_canonical() {
        let mut input = normalized_document();
        input["product"]["variants"]
            .as_array_mut()
            .unwrap()
            .truncate(1);

        assert_eq!(
            structuralized(&input)["variations"][0]["url"],
            "https://www.ankeritaly.com/products/a121c"
        );
    }

    #[test]
    fn an_unusable_identifier_crosses_as_the_identifier_failure() {
        let mut input = normalized_document();
        input["product"]["variants"][0]["sku"] = json!("");

        assert_eq!(
            error(input),
            StructuralizeError::Identifier(IdentifierError::EmptySku)
        );
    }
}
