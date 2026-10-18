use super::super::raw;
use super::{
    NormalizeError,
    offers::{amount, invalid, require},
};
use money::{Currency, Money};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Product {
    pub description: Option<String>,
    /// The analytics `gid`.
    pub id: String,
    pub handle: String,
    /// The page canonical URL.
    pub url: String,
    /// The main section's `h1`.
    pub title: String,
    /// The analytics vendor.
    pub vendor: String,
    /// `None` where the product is untyped.
    pub product_type: Option<String>,
    // Commented out: false on every page.
    // pub remote: bool,
    // Commented out: the analytics `page` describes this request, not the product: a product
    // page of `resourceId` the product id, under a fresh `requestId`.
    // pub page: Page,
    /// The variant picker's legend titles without their trailing colon. Empty without a picker.
    pub options: Vec<String>,
    /// The main gallery's image slides, in slide order, without the display `width` the theme
    /// asks the CDN for. Video slides are left out: they render only poster thumbnails.
    pub images: Vec<String>,
    // Commented out: the product title or a localized caption, nothing a variation needs.
    // pub image_alts: Vec<Option<String>>,
    /// Every variant, in the analytics' order.
    pub variants: Vec<Variant>,
    pub selected_variant: SelectedVariant,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Variant {
    /// Price published by the selling JSON-LD offer, which can reflect an automatic discount.
    pub offer_price: Option<Money>,
    pub offer_prices: Vec<OfferPrice>,
    pub condition: Option<String>,
    pub url: String,
    pub available: Option<bool>,
    pub gtin: Option<String>,
    pub image: Option<String>,
    pub compare_at_price: Option<Money>,
    pub id: String,
    /// The selected values joined with ` / `. `None` on a product without options.
    pub title: Option<String>,
    /// The product title and `title`, joined with ` - `, or the product title alone.
    pub name: String,
    pub sku: Option<String>,
    /// Before any automatic discount; see `SelectedVariant::automatic_discount_price`.
    pub price: Money,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SelectedVariant {
    pub id: String,
    /// Stock state from the matching JSON-LD offer.
    pub available: Option<bool>,
    /// The struck-through price, where the variant is marked down. `None` also where an
    /// automatic discount shows: the theme then strikes through the variant price instead, and
    /// the markdown, if any, is not rendered.
    pub compare_at_price: Option<Money>,
    /// The price after an automatic discount (a storefront app's, not a code), where one applies.
    pub automatic_discount_price: Option<Money>,
    /// From the picker's record; `None` without a picker.
    pub barcode: Option<String>,
    // Commented out: `price` and `public_title` restate the analytics variant; `available`
    // restates the add-to-cart button.
    // Commented out: `requires_shipping` and `taxable` are true on every record.
    // Commented out: the variant's image is one of the gallery's.
    // pub featured_image: Option<FeaturedImage>,
    // pub featured_media: Option<FeaturedMedia>,
    // Commented out: `shopify` on every record; stock is tracked, but no quantity is rendered.
    // pub inventory_management: Option<String>,
    // Commented out: one unit, no maximum, in steps of one, on every record.
    // pub quantity_rule: QuantityRule,
    // Commented out: no selling plans on any page; `requires_selling_plan` is false throughout.
    // pub selling_plan_allocations: Vec<Value>,
    // Commented out: 0, unweighed, on every record.
    // pub weight: i64,
    /// The picker's record's values paired with the legends. Empty without a picker.
    pub options: Vec<VariantOption>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VariantOption {
    pub name: String,
    pub value: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OfferPrice {
    pub kind: String,
    pub price: Money,
}

pub(super) fn gallery_image(src: &str) -> String {
    let url = match src.strip_prefix("//") {
        Some(rest) => format!("https://{rest}"),
        None => src.to_string(),
    };

    let Some((path, query)) = url.split_once('?') else {
        return url;
    };
    let kept: Vec<&str> = query
        .split('&')
        .filter(|pair| !pair.starts_with("width="))
        .collect();

    if kept.is_empty() {
        path.to_string()
    } else {
        format!("{path}?{}", kept.join("&"))
    }
}

fn won_text(text: &str) -> Result<Money, NormalizeError> {
    let decimal: String = text
        .chars()
        .filter(|c| !c.is_whitespace() && *c != ',' && *c != '₩')
        .collect();
    amount(&decimal, Currency::KRW)
}
fn variant_gid(id: u64) -> String {
    format!("gid://shopify/ProductVariant/{id}")
}
impl Product {
    pub(crate) fn from_raw(raw: raw::UgreenKrRawDocument) -> Result<Self, NormalizeError> {
        let meta = raw.analytics.meta.product;
        require(
            raw.analytics.meta.page.resource_id == meta.id,
            "page.resourceId",
            "does not match product",
        )?;
        require(
            meta.gid == format!("gid://shopify/Product/{}", meta.id),
            "product.gid",
            "does not match numeric id",
        )?;
        require(
            raw.canonical.href == format!("https://kr.ugreen.com/products/{}", meta.handle),
            "canonical",
            "does not match analytics handle",
        )?;
        let selected: u64 = raw
            .selected_variant_id
            .value
            .parse()
            .map_err(|_| invalid("selected_variant_id", "not a numeric id"))?;
        let mut variants = Vec::new();
        for v in meta.variants {
            require(
                v.price % 100 == 0,
                "variants.price",
                "hundredths do not represent whole won",
            )?;
            variants.push(Variant {
                offer_price: None,
                id: variant_gid(v.id),
                title: v.public_title,
                name: v.name,
                sku: v.sku.filter(|s| !s.is_empty()),
                price: Money::new(v.price / 100, Currency::KRW),
                url: String::new(),
                available: None,
                gtin: None,
                image: None,
                compare_at_price: None,
                condition: None,
                offer_prices: Vec::new(),
            });
        }
        let listed = variants
            .iter()
            .find(|v| v.id == variant_gid(selected))
            .ok_or_else(|| invalid("selected_variant_id", "absent from analytics"))?
            .price
            .clone();
        let displayed = won_text(&raw.displayed_price.text)?;
        let compare = if raw.compare_price.text.trim().is_empty() {
            None
        } else {
            Some(won_text(&raw.compare_price.text)?)
        };
        require(
            displayed.amount() <= listed.amount(),
            "displayed_price",
            "above analytics price",
        )?;
        let discounted = displayed.amount() < listed.amount();
        if discounted {
            require(
                compare.as_ref() == Some(&listed),
                "compare_price",
                "discount is not from analytics price",
            )?;
        }
        let selected_variant = SelectedVariant {
            id: variant_gid(selected),
            available: None,
            barcode: None,
            options: Vec::new(),
            compare_at_price: if discounted {
                None
            } else {
                compare.filter(|p| p.amount() > listed.amount())
            },
            automatic_discount_price: discounted.then_some(displayed),
        };
        let mut images = Vec::new();
        for img in raw.images {
            let src = gallery_image(&img.src);
            if !images.contains(&src) {
                images.push(src);
            }
        }
        let mut product = Self {
            id: meta.gid,
            handle: meta.handle,
            url: raw.canonical.href,
            title: raw.title.text,
            description: None,
            vendor: meta.vendor,
            product_type: (!meta.product_type.is_empty()).then_some(meta.product_type),
            options: Vec::new(),
            images,
            variants,
            selected_variant,
        };
        super::offers::enrich(&mut product, raw.ld_json)?;
        Ok(product)
    }
}
