ALTER TABLE raw_offers RENAME TO offer_details_raw;
ALTER TABLE extracted_offers RENAME TO offer_details_extracted;
ALTER TABLE normalized_offers RENAME TO offer_details_normalized;
ALTER TABLE structuralized_offers RENAME TO offer_details_structuralized;
ALTER TABLE raw_sitemap_documents RENAME TO sitemaps_raw;
ALTER TABLE processed_sitemaps RENAME TO sitemaps_processed;
ALTER TABLE consolidated_sitemaps RENAME TO sitemaps_consolidated;
