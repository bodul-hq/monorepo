-- Immutable exports with durable upload preparation, separate from current-stage metadata.
CREATE TABLE offer_export (
    id UUID PRIMARY KEY,
    offer_id UUID NOT NULL,
    source_object_id UUID NOT NULL,
    schema_version INTEGER NOT NULL CHECK (schema_version > 0),
    source JSONB NOT NULL CHECK (jsonb_typeof(source) = 'object'),
    stats JSONB NOT NULL CHECK (jsonb_typeof(stats) = 'object'),
    status TEXT NOT NULL DEFAULT 'prepared',
    prepared_body TEXT,
    storage JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    completed_at TIMESTAMPTZ,
    UNIQUE (offer_id, source_object_id, schema_version),
    CHECK (
        (status = 'prepared' AND prepared_body IS NOT NULL AND storage IS NULL AND completed_at IS NULL)
        OR
        (status = 'completed' AND prepared_body IS NULL AND storage IS NOT NULL
         AND jsonb_typeof(storage) = 'object' AND completed_at IS NOT NULL)
    )
);
