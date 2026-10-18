//! What the architecture reads from the main product section's rendered DOM: the variant
//! picker's JSON, the selected variant, the price block and the gallery.

use html_extractor::serde_json::Value;

/// `variant-radios script[type="application/json"]`: Liquid's variants, rendered only where the
/// product has a picker (69 of the 132 saved pages; the 63 `Default Title` products have none).
#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VariantsScript {
    #[serde(rename = "type")]
    pub script_type: String,
    pub data: Vec<PickerVariant>,
}

/// Liquid's variant. Every amount is in hundredths of a yen, as in the analytics.
#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PickerVariant {
    pub id: u64,
    pub title: String,
    pub option1: Option<String>,
    pub option2: Option<String>,
    pub option3: Option<String>,
    /// `null` on 13 of the 100 picker variants.
    pub sku: Option<String>,
    pub requires_shipping: bool,
    pub taxable: bool,
    pub featured_image: Option<FeaturedImage>,
    pub available: bool,
    pub name: String,
    pub public_title: Option<String>,
    pub options: Vec<String>,
    pub price: i64,
    /// Unit unknown, Liquid documents grams. `0`, unweighed, on 99 of the 100 picker variants.
    pub weight: i64,
    /// `null` on 91 of the 100 picker variants; `0` on one and `price` itself on 8.
    pub compare_at_price: Option<i64>,
    pub inventory_management: Option<String>,
    /// `null` on every saved variant.
    pub barcode: Option<String>,
    /// Only rendered alongside a `featured_image`.
    #[serde(default)]
    pub featured_media: Option<FeaturedMedia>,
    pub requires_selling_plan: bool,
    pub selling_plan_allocations: Vec<Value>,
    pub quantity_rule: QuantityRule,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FeaturedImage {
    pub id: u64,
    pub product_id: u64,
    pub position: i64,
    pub created_at: String,
    pub updated_at: String,
    pub alt: Option<String>,
    pub width: i64,
    pub height: i64,
    /// Protocol-relative, such as `//ugreen.jp/cdn/shop/files/a.webp?v=1`.
    pub src: String,
    pub variant_ids: Vec<u64>,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FeaturedMedia {
    pub alt: Option<String>,
    pub id: u64,
    pub position: i64,
    pub preview_image: PreviewImage,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PreviewImage {
    pub aspect_ratio: f64,
    pub height: i64,
    pub width: i64,
    pub src: String,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct QuantityRule {
    pub min: i64,
    pub max: Option<i64>,
    pub increment: i64,
}

/// The cart form's `input[name="id"]`: the variant the price block is rendered for.
#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SelectedVariantId {
    /// A numeric variant id, as a string.
    pub value: String,
}

/// Dawn's price snippet, its texts rendered in yen with grouping commas (`¥65,880`).
#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PriceBlock {
    /// `price price--product`, plus `price--on-sale` where a compare-at price is shown.
    pub class: String,
    pub regular: PriceText,
    pub sale: PriceText,
    /// The struck-through price under `.price__sale`. Off sale it is empty (94 pages), repeats
    /// the price (35) or reads `¥0` (1).
    pub compare_at: PriceText,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PriceText {
    pub text: String,
}

/// A gallery slide. Every slide on the saved pages is an image.
#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MediaSlide {
    /// The media id prefixed with the section id (`template--…__main-44439502291224`).
    pub id: String,
    /// Protocol-relative, at the theme's display width
    /// (`//ugreen.jp/cdn/shop/files/a.png?v=1&width=1946`).
    pub src: String,
    pub alt: String,
}

#[cfg(test)]
pub(crate) mod tests {
    use super::{MediaSlide, PickerVariant, PriceBlock, VariantsScript};
    use html_extractor::serde_json::{self, Value, json};

    pub(crate) fn picker_variant(id: u64, price: i64, sku: &str, length: &str) -> Value {
        json!({
            "id": id,
            "title": length,
            "option1": length,
            "option2": null,
            "option3": null,
            "sku": sku,
            "requires_shipping": false,
            "taxable": false,
            "featured_image": null,
            "available": true,
            "name": format!("UGREENライトニング USB ケーブル - {length}"),
            "public_title": length,
            "options": [length],
            "price": price,
            "weight": 0,
            "compare_at_price": null,
            "inventory_management": null,
            "barcode": null,
            "requires_selling_plan": false,
            "selling_plan_allocations": [],
            "quantity_rule": {"min": 1, "max": null, "increment": 1}
        })
    }

    /// The picker of `products-lightning-usb-cable.html`.
    pub(crate) fn variants() -> Value {
        json!({
            "type": "application/json",
            "data": [
                picker_variant(49142586999064, 139900, "60521", "1Ｍ"),
                picker_variant(49154830631192, 159900, "70733", "2M")
            ]
        })
    }

    pub(crate) fn price(class: &str, price: &str, compare_at: &str) -> Value {
        json!({
            "class": class,
            "regular": {"text": price},
            "sale": {"text": price},
            "compare_at": {"text": compare_at}
        })
    }

    pub(crate) fn media() -> Value {
        json!([
            {"id": "template--22963901956376__main-39252486521112",
                "src": "//ugreen.jp/cdn/shop/files/60521-1.png?v=1726277735&width=1946", "alt": ""},
            {"id": "template--22963901956376__main-39252486488344",
                "src": "//ugreen.jp/cdn/shop/files/60521-2.png?v=1726277735&width=1946", "alt": ""}
        ])
    }

    #[test]
    fn the_picker_deserializes() {
        let script = serde_json::from_value::<VariantsScript>(variants()).expect("deserializes");
        assert_eq!(script.data.len(), 2);
        assert_eq!(script.data[1].price, 159900);
    }

    #[test]
    fn a_variant_with_its_own_image_deserializes() {
        let mut variant = picker_variant(52043775312152, 499000, "PB570", "ブラック");
        variant["featured_image"] = json!({
            "id": 52890795082008_u64, "product_id": 10262604677400_u64, "position": 1,
            "created_at": "2026-05-11T18:06:49+09:00", "updated_at": "2026-05-11T18:07:41+09:00",
            "alt": null, "width": 1200, "height": 1200,
            "src": "//ugreen.jp/cdn/shop/files/PB570_65839.webp?v=1778490461",
            "variant_ids": [52043775312152_u64]
        });
        variant["featured_media"] = json!({
            "alt": null, "id": 43799982604568_u64, "position": 1,
            "preview_image": {"aspect_ratio": 1.0, "height": 1200, "width": 1200,
                "src": "//ugreen.jp/cdn/shop/files/PB570_65839.webp?v=1778490461"}
        });

        let variant = serde_json::from_value::<PickerVariant>(variant).expect("deserializes");
        assert!(variant.featured_image.is_some());
        assert!(variant.featured_media.is_some());
    }

    #[test]
    fn an_unknown_key_in_the_picker_fails() {
        let mut input = variants();
        input["data"][0]["unit_price"] = json!(1);
        assert!(serde_json::from_value::<VariantsScript>(input).is_err());

        let mut attribute = variants();
        attribute["id"] = json!("variants");
        assert!(serde_json::from_value::<VariantsScript>(attribute).is_err());
    }

    #[test]
    fn the_price_block_deserializes_and_rejects_a_new_text() {
        let block = serde_json::from_value::<PriceBlock>(price(
            "price price--product price--on-sale",
            "¥52,700",
            "¥65,880",
        ))
        .expect("deserializes");
        assert_eq!(block.compare_at.text, "¥65,880");

        let mut input = price("price price--product", "¥1,399", "");
        input["unit"] = json!({"text": ""});
        assert!(serde_json::from_value::<PriceBlock>(input).is_err());
    }

    #[test]
    fn a_slide_deserializes_and_rejects_a_new_attribute() {
        let slides = serde_json::from_value::<Vec<MediaSlide>>(media()).expect("deserializes");
        assert_eq!(slides.len(), 2);

        let mut input = media();
        input[0]["data-media-type"] = json!("video");
        assert!(serde_json::from_value::<Vec<MediaSlide>>(input).is_err());
    }
}
