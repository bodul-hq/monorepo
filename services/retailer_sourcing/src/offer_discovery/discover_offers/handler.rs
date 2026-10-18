use uuid::Uuid;

use super::command::DiscoverOffers;
use super::entity::ConsolidatedSitemap;
use super::event::OfferDiscovered;
use super::store::StoreError;
use crate::platform::bus::Handle;

/// The storage this slice needs.
pub trait Store: Send + Sync + 'static {
    /// The product URLs of a consolidated sitemap.
    fn consolidated_sitemap(&self, id: Uuid) -> Result<ConsolidatedSitemap, StoreError>;
}

pub struct Handler<S> {
    store: S,
}

impl<S> Handler<S> {
    pub fn new(store: S) -> Self {
        Self { store }
    }
}

impl<S: Store> Handle for Handler<S> {
    type Command = DiscoverOffers;
    type Event = OfferDiscovered;
    type Error = StoreError;

    fn handle(&self, command: DiscoverOffers) -> Result<Vec<OfferDiscovered>, StoreError> {
        let DiscoverOffers {
            consolidated_sitemap_id,
            retailer_code,
        } = command;

        let sitemap = self.store.consolidated_sitemap(consolidated_sitemap_id)?;
        tracing::debug!(object_storage_id = %sitemap.object_storage_id, "input object loaded");
        let urls = sitemap.product_urls;

        tracing::info!(
            %consolidated_sitemap_id,
            ?retailer_code,
            offers = urls.len(),
            "offers discovered"
        );
        Ok(urls
            .into_iter()
            .map(|url| OfferDiscovered {
                offer_id: Uuid::now_v7(),
                consolidated_sitemap_id: sitemap.id,
                retailer_code,
                url,
            })
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use bodul_shared::retailer::RetailerCode;

    use super::*;

    struct FakeStore {
        urls: Vec<String>,
    }

    impl Store for FakeStore {
        fn consolidated_sitemap(&self, id: Uuid) -> Result<ConsolidatedSitemap, StoreError> {
            Ok(ConsolidatedSitemap {
                object_storage_id: Uuid::now_v7(),
                id,
                product_urls: self.urls.clone(),
            })
        }
    }

    fn handler(urls: &[&str]) -> Handler<FakeStore> {
        Handler::new(FakeStore {
            urls: urls.iter().map(|url| url.to_string()).collect(),
        })
    }

    #[test]
    fn discovers_an_offer_per_product_url() {
        let handler = handler(&[
            "https://shop.test/products/a",
            "https://shop.test/products/b",
        ]);
        let consolidated_sitemap_id = Uuid::now_v7();

        let events = handler
            .handle(DiscoverOffers {
                consolidated_sitemap_id,
                retailer_code: RetailerCode::UgreenCa,
            })
            .unwrap();

        let urls: Vec<&str> = events.iter().map(|event| event.url.as_str()).collect();
        assert_eq!(
            urls,
            [
                "https://shop.test/products/a",
                "https://shop.test/products/b"
            ]
        );
        assert!(events.iter().all(|event| {
            event.consolidated_sitemap_id == consolidated_sitemap_id
                && event.retailer_code == RetailerCode::UgreenCa
        }));
        assert_ne!(events[0].offer_id, events[1].offer_id);
    }

    #[test]
    fn discovers_nothing_without_product_urls() {
        let events = handler(&[])
            .handle(DiscoverOffers {
                consolidated_sitemap_id: Uuid::now_v7(),
                retailer_code: RetailerCode::UgreenCa,
            })
            .unwrap();

        assert!(events.is_empty());
    }
}
