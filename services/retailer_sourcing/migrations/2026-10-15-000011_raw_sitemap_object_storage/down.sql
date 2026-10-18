-- HTTP objects cannot be restored by a database migration. Refuse a rollback
-- that would leave object-backed rows unreadable by the previous service.
DO $$ BEGIN
    IF EXISTS (SELECT 1 FROM sitemaps_raw) THEN
        RAISE EXCEPTION 'Remove new raw sitemap rows before reverting the object-storage cutover';
    END IF;
END $$;
CREATE TABLE raw_sitemap_documents_data (
    id UUID PRIMARY KEY REFERENCES sitemaps_raw(id) ON DELETE CASCADE,
    body TEXT NOT NULL
);
