//! Strict extracted document. Optional picker records are absent on single-variant pages.

pub mod analytics;
pub mod ldjsonitem;
pub mod variant;

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UgreenUkRawDocument {
    #[serde(rename = "ld-json")]
    pub ld_json: Vec<ldjsonitem::LdJsonItem>,
    pub analytics: analytics::AnalyticsScript,
    pub title: Title,
    pub price: PriceBlock,
    pub product_form: ProductForm,
    pub selected_variant_id: SelectedVariantId,
    pub add_to_cart: AddToCart,
    /// Only rendered when the product has options to choose between.
    pub variant_picker: Option<VariantPicker>,
    pub option_names: Vec<OptionName>,
    /// The variant picker's record of the selected variant; absent without a picker.
    pub selected_variant: Option<variant::VariantScript>,
    pub images: Vec<Image>,
}

/// The main product section's `h1`.
#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Title {
    pub text: String,
}

/// The main `product-price` element's data attributes, all strings. They describe the selected
/// variant as the theme displays it, after any automatic discount.
#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PriceBlock {
    pub product_id: String,
    pub base_price_cents: String,
    pub base_compare_cents: String,
    pub currency: String,
}

/// The main product form, not the sticky bar's copy of it.
#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProductForm {
    pub product_id: String,
    pub product_url: String,
}

/// The main product form's hidden `id` input.
#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SelectedVariantId {
    pub value: String,
}

/// The main product form's add-to-cart button.
#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AddToCart {
    /// Present, and empty, where the selected variant is sold out.
    pub disabled: Option<String>,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VariantPicker {
    pub product_id: String,
    pub product_url: String,
}

/// A picker fieldset's legend title, such as `Länge:`, in the order of the variants' option
/// slots.
#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OptionName {
    pub text: String,
}

/// An image slide of the main media gallery; video slides are left out.
#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Image {
    pub src: String,
    pub alt: Option<String>,
}
