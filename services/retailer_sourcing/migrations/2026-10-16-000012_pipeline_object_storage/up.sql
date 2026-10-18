-- Fresh-data cutover. Raw sitemaps and bus history remain intact.
DELETE FROM sitemaps_processed;
DROP TABLE processed_sitemaps_data;
DELETE FROM sitemaps_consolidated;
DROP TABLE consolidated_sitemaps_data;
DELETE FROM offer_details_raw;
DROP TABLE raw_offers_data;
DELETE FROM offer_details_extracted;
DROP TABLE extracted_offers_data;
DELETE FROM offer_details_normalized;
DROP TABLE normalized_offers_data;
DELETE FROM offer_details_structuralized;
DROP TABLE structuralized_offers_data;
