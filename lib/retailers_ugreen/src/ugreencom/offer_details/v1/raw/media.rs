/// A main gallery item. All 72 items on the saved pages are images; the gallery renders no video
/// or model, so one fails here until it is typed.
#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(tag = "media_type", rename_all = "snake_case", deny_unknown_fields)]
pub enum MediaItem {
    Image {
        /// The bare media id, digits as a string.
        id: String,
        /// Protocol-relative, ending in the `&width=1445` display width.
        src: String,
        alt: String,
    },
}

#[cfg(test)]
pub(crate) mod tests {
    use super::MediaItem;
    use html_extractor::serde_json::{self, Value, json};

    pub(crate) fn image() -> Value {
        json!({
            "alt": "UGREEN Revodok 105 USB C Hub 5 in 1 Multiport Adapter",
            "id": "39249312612571",
            "media_type": "image",
            "src": "//www.ugreen.com/cdn/shop/files/d0a9824bf408d4d39de70224bbafb22d.png?v=1762322082&width=1445"
        })
    }

    #[test]
    fn an_image_deserializes() {
        let MediaItem::Image { id, .. } =
            serde_json::from_value::<MediaItem>(image()).expect("deserializes");
        assert_eq!(id, "39249312612571");
    }

    #[test]
    fn an_unseen_media_type_fails() {
        for media_type in ["video", "external_video", "model"] {
            let mut item = image();
            item["media_type"] = json!(media_type);
            let error = serde_json::from_value::<MediaItem>(item).expect_err("only images");
            assert!(
                error
                    .to_string()
                    .contains(&format!("unknown variant `{media_type}`"))
            );
        }
    }

    #[test]
    fn a_new_key_fails() {
        let mut item = image();
        item["width"] = json!("1445");

        assert!(serde_json::from_value::<MediaItem>(item).is_err());
    }
}
