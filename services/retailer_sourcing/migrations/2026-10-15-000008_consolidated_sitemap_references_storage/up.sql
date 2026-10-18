-- Keep the source references and storage details in JSONB only.
ALTER TABLE consolidated_sitemaps
    ADD COLUMN "references" JSONB,
    ADD COLUMN storage JSONB;
UPDATE consolidated_sitemaps SET
    "references" = jsonb_build_object(
        'processed_sitemap_id', processed_sitemap_id,
        'retrieval_id', retrieval_id,
        'retailer_code', retailer_code
    ),
    storage = jsonb_build_object('content_size', content_size);
ALTER TABLE consolidated_sitemaps
    ALTER COLUMN "references" SET NOT NULL,
    ALTER COLUMN storage SET NOT NULL,
    DROP COLUMN processed_sitemap_id,
    DROP COLUMN retrieval_id,
    DROP COLUMN retailer_code,
    DROP COLUMN content_size;

-- UUID comparison retains the original uniqueness semantics without a foreign key.
CREATE UNIQUE INDEX consolidated_sitemaps_processed_reference_key
    ON consolidated_sitemaps ((("references" ->> 'processed_sitemap_id')::UUID));
