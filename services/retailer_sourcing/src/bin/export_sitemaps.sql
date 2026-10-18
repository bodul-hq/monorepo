SELECT DISTINCT ON (p.retailer_code)
        p.id, p.retailer_code, p.retrieval_id, p.storage,
        c.id AS consolidated_id, c.storage AS consolidated_storage
    FROM sitemaps_processed p
    LEFT JOIN sitemaps_consolidated c ON (c."references" ->> 'processed_sitemap_id')::UUID = p.id
    ORDER BY p.retailer_code, p.processed_at DESC
