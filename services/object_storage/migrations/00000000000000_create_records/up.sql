CREATE TABLE data (
    id uuid PRIMARY KEY,
    payload text NOT NULL
);

CREATE TABLE records (
    id uuid PRIMARY KEY,
    bucket text NOT NULL,
    external_id uuid NOT NULL,
    data_id uuid NOT NULL REFERENCES data(id),
    metadata jsonb NOT NULL,
    created_at timestamptz NOT NULL,
    CONSTRAINT records_bucket_external_id UNIQUE (bucket, external_id)
);
