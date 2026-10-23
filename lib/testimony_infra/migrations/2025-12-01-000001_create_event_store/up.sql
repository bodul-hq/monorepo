CREATE TABLE event_store (
    id BIGSERIAL PRIMARY KEY,
    agg_id VARCHAR(255) NOT NULL,
    sequence BIGINT NOT NULL,
    payload_type VARCHAR(255) NOT NULL,
    payload JSONB NOT NULL,
    metadata JSONB NOT NULL,
    recorded_on TIMESTAMPTZ NOT NULL,

    CONSTRAINT unique_agg_sequence UNIQUE (agg_id, sequence)
);

CREATE INDEX idx_event_store_agg_id ON event_store(agg_id);
CREATE INDEX idx_event_store_recorded_on ON event_store(recorded_on);
