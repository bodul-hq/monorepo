# retailers_wip

Work-in-progress retailer integrations. The crate has no code yet.

The structuralized Shopify document it used to hold now lives in
`retailer_sauce::shopify::structuralized`. Retailer crates expose it through
`structuralize_offer_details_v2_for`, and
`retailer_data_extractor::structuralize_v2_for` dispatches to them.
