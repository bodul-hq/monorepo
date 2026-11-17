# Initial offer fill

Downloads a retailer's full Shopify catalog, per supported locale, and stores raw
product details. Normalization and publishing are separate.

## Steps

1. Start a run per retailer with `POST /api/initial-offer-fills`.
2. Try to acquire a Storefront API token.
3. For each locale in the retailer entity, page through the catalog for product IDs.
4. Fetch each product's details (variants, prices, images) in that locale.
5. Store the raw details in object storage.

## Options today

- **Retailers:** 48 stores: 17 Anker, 10 Minisforum, 21 Ugreen.
- **Token:** enables catalog and product-detail API access. Required for most
  Anker stores; other stores can continue without one.
- **Locales:** define the region and language where offers are eligible. Some
  retailers require an explicit locale. Defined in the retailer entity, per
  [retailer.md](../retailer.md).
- **Failures:** retried by the queue.
