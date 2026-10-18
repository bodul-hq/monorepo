/// The main section's "Shop Now" button, the page's only call to action: it links out to the
/// regional store that sells the product. Missing on `products-us-85185.html`, whose product has
/// no sales channel.
#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ShopNow {
    pub class: String,
    /// `USA` on all 65 buttons.
    #[serde(rename = "data-market")]
    pub market: String,
    #[serde(rename = "data-module")]
    pub module: String,
    /// The analytics variant's SKU on all 65 buttons.
    #[serde(rename = "data-product-sku")]
    pub product_sku: String,
    /// The JSON that `channels` is parsed from, verbatim.
    #[serde(rename = "data-product-url")]
    pub product_url: String,
    #[serde(rename = "data-track-event")]
    pub track_event: String,
    /// Tracking JSON naming the button and the SKU again.
    #[serde(rename = "data-track-event-parameters")]
    pub track_event_parameters: String,
    pub channels: Channels,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Channels {
    /// One link on all 65 buttons: `us.ugreen.com` on 54, `ai.ugreen.com` on 11.
    pub channel_url_list: Vec<String>,
}

#[cfg(test)]
pub(crate) mod tests {
    use super::ShopNow;
    use html_extractor::serde_json::{self, Value, json};

    /// From `products-usa-15495.html`.
    pub(crate) fn button() -> Value {
        json!({
            "channels": {"channel_url_list": ["https://us.ugreen.com/products/15495"]},
            "class": "ug-product-buy-button",
            "data-market": "USA",
            "data-module": "Product Detail Primary",
            "data-product-sku": "usa-15495",
            "data-product-url": "{\"channel_url_list\":[\"https://us.ugreen.com/products/15495\"]}",
            "data-track-event": "click_button",
            "data-track-event-parameters": "{\"module\":\"Product Detail Primary Button\", \"button_name\": \"Shop Now\",\"product_sku\":\"usa-15495\"}"
        })
    }

    #[test]
    fn the_button_deserializes() {
        let button = serde_json::from_value::<ShopNow>(button()).expect("deserializes");
        assert_eq!(
            button.channels.channel_url_list,
            ["https://us.ugreen.com/products/15495"]
        );
    }

    #[test]
    fn a_new_attribute_fails() {
        let mut input = button();
        input["data-variant-id"] = json!("1");

        assert!(serde_json::from_value::<ShopNow>(input).is_err());
    }

    #[test]
    fn a_new_key_in_the_channels_fails() {
        let mut input = button();
        input["channels"]["default"] = json!("https://us.ugreen.com/products/15495");

        assert!(serde_json::from_value::<ShopNow>(input).is_err());
    }
}
