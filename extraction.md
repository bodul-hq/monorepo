# Shopify extraction notes

## Extracted offer details

The main structure is named **extracted offer details**. It gathers product,
offer, and offer-related data in one payload.

The acquisition time (`acquired_at`) is stored at the root. It records when the
data was fetched into our system, because this time is not stored in any of
the events.

## Context

The context records the locale, country, currency, and language represented
by the payload.

## Product and related data

The payload is split into product, collections, images, and media. Collections,
images, and media are extracted into separate collections alongside the product
to reduce duplication and organize the volume of related information.

The product stores only their IDs as references.

Product SEO title and description are stored as `product.content.seo_title`
and `product.content.seo_description`. Both retain source nulls. Collection SEO
remains in its existing `seo` object.

## Extraction rules

- **Unavailable category:** Set the category to `None` instead of retaining
  an unavailable category in the extracted data.
- **Single-variant placeholder options:** When a Shopify product has exactly
  one option and one variant, and uses the placeholder option name `Title`
  and value `Default Title`, remove those placeholders. The product has only
  one variant and does not need options to distinguish variants.
- **Empty unit price measurement:** A unit price measurement can arrive as an
  object containing zeros and nulls. Set this empty measurement to `None`
  in the extracted data.
- **Variant counts:** Use variant counts to check variants during conversion,
  but do not store the counts in the extracted data.
- **Zero price:** Treat a price of zero as an unknown price, meaning no price
  is available.
