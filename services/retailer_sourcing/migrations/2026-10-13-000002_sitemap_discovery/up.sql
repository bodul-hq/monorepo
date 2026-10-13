-- Sitemap discovery's tables. The rows of one crawl share a retrieval_id.

-- Each sitemap file as fetched.
CREATE TABLE raw_sitemap_documents (
    id UUID PRIMARY KEY,
    retrieval_id UUID NOT NULL,
    url TEXT NOT NULL,
    last_modified TIMESTAMPTZ,
    body TEXT NOT NULL,
    body_size INT NOT NULL,
    fetched_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (retrieval_id, url)
);

-- A crawl's files parsed into one sitemap tree.
CREATE TABLE processed_sitemaps (
    id UUID PRIMARY KEY,
    retrieval_id UUID NOT NULL UNIQUE,
    retailer_code TEXT NOT NULL,
    document JSONB NOT NULL,
    document_size INT NOT NULL,
    url_count INT NOT NULL,
    processed_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- A processed sitemap's URLs, grouped by kind of page.
CREATE TABLE consolidated_sitemaps (
    id UUID PRIMARY KEY,
    processed_sitemap_id UUID NOT NULL UNIQUE REFERENCES processed_sitemaps(id) ON DELETE CASCADE,
    retrieval_id UUID NOT NULL,
    retailer_code TEXT NOT NULL,
    content JSONB NOT NULL,
    content_size INT NOT NULL,
    product_count INT NOT NULL,
    catalog_count INT NOT NULL,
    content_count INT NOT NULL,
    not_interested_count INT NOT NULL,
    unknown_count INT NOT NULL,
    consolidated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
