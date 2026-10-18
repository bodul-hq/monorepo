/// The theme's `data-product-featured` attribute on the main product section: its settings for
/// the section, plus the product-level price, strikethrough price and availability. On a
/// `Default Title` product, which renders no variant records, those are its one variant's.
#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FeaturedAttribute {
    pub data: Featured,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Featured {
    /// The product id, as a string.
    pub id: String,
    #[serde(rename = "isMainProduct")]
    pub is_main_product: bool,
    #[serde(rename = "sectionId")]
    pub section_id: String,
    #[serde(rename = "disableSwatch")]
    pub disable_swatch: bool,
    pub media: bool,
    #[serde(rename = "enableHistoryState")]
    pub enable_history_state: bool,
    #[serde(rename = "formID")]
    pub form_id: String,
    #[serde(rename = "removeSoldout")]
    pub remove_soldout: bool,
    #[serde(rename = "changeVariantByImg")]
    pub change_variant_by_img: bool,
    #[serde(rename = "isNoPick")]
    pub is_no_pick: bool,
    #[serde(rename = "hasSoldoutUnavailable")]
    pub has_soldout_unavailable: bool,
    #[serde(rename = "canMediaGroup")]
    pub can_media_group: bool,
    #[serde(rename = "isGrouped")]
    pub is_grouped: bool,
    #[serde(rename = "hasIsotope")]
    pub has_isotope: bool,
    pub available: bool,
    #[serde(rename = "customBadge")]
    pub custom_badge: Option<String>,
    #[serde(rename = "customBadgeHandle")]
    pub custom_badge_handle: Option<String>,
    /// Unix seconds.
    #[serde(rename = "dateStart")]
    pub date_start: i64,
    pub compare_at_price: Option<i64>,
    pub price: i64,
    /// The theme's spelling.
    #[serde(rename = "isPreoder")]
    pub is_preorder: bool,
    #[serde(rename = "showFirstMedia")]
    pub show_first_media: bool,
    #[serde(rename = "isSticky")]
    pub is_sticky: bool,
    #[serde(rename = "isStickyMB")]
    pub is_sticky_mb: bool,
    #[serde(rename = "stickyShow")]
    pub sticky_show: String,
}

#[cfg(test)]
pub(crate) mod tests {
    use super::Featured;
    use html_extractor::serde_json::{self, Value, json};

    pub(crate) fn data() -> Value {
        json!({
            "id": "8006139642144",
            "isMainProduct": true,
            "sectionId": "template--28291008889120__main",
            "disableSwatch": false,
            "media": true,
            "enableHistoryState": true,
            "formID": "#product-form-8006139642144template--28291008889120__main",
            "removeSoldout": false,
            "changeVariantByImg": true,
            "isNoPick": false,
            "hasSoldoutUnavailable": false,
            "canMediaGroup": false,
            "isGrouped": false,
            "hasIsotope": false,
            "available": true,
            "customBadge": null,
            "customBadgeHandle": null,
            "dateStart": 1668752494,
            "compare_at_price": 7999,
            "price": 5999,
            "isPreoder": false,
            "showFirstMedia": false,
            "isSticky": true,
            "isStickyMB": true,
            "stickyShow": "1"
        })
    }

    #[test]
    fn the_attribute_deserializes() {
        let featured = serde_json::from_value::<Featured>(data()).expect("deserializes");
        assert_eq!(featured.price, 5999);
    }
}
