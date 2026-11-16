-- Tables used by the Mulac kernel (mulac-kernel 0.5.0) to queue commands and
-- events. Their columns must match the Diesel schemas inside Mulac's crates.

CREATE TABLE inbox_entries (
    id UUID PRIMARY KEY,
    status INT NOT NULL DEFAULT 0,
    payload TEXT NOT NULL,
    meta JSONB NOT NULL,
    scheduled_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    attempts INT NOT NULL DEFAULT 0,
    reservation_id UUID,
    reserved_at TIMESTAMPTZ,
    received_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    processed_at TIMESTAMPTZ,
    extra_info JSONB
);
CREATE INDEX idx_inbox_entries_status_scheduled_at ON inbox_entries (status, scheduled_at);
CREATE INDEX idx_inbox_entries_status_reserved_at ON inbox_entries (status, reserved_at);

CREATE TABLE command_entries (
    id UUID PRIMARY KEY,
    command_type TEXT NOT NULL,
    status INT NOT NULL DEFAULT 0,
    payload TEXT NOT NULL,
    meta JSONB,
    scheduled_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    attempts INT NOT NULL DEFAULT 0,
    reservation_id UUID,
    reserved_at TIMESTAMPTZ,
    received_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    processed_at TIMESTAMPTZ,
    extra_info JSONB
);
CREATE INDEX idx_command_entries_status_scheduled_at ON command_entries (status, scheduled_at);
CREATE INDEX idx_command_entries_reservation_id ON command_entries (reservation_id);

CREATE TABLE event_entries (
    id UUID PRIMARY KEY,
    event_type TEXT NOT NULL,
    status INT NOT NULL DEFAULT 0,
    payload TEXT NOT NULL,
    meta JSONB,
    scheduled_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    attempts INT NOT NULL DEFAULT 0,
    reservation_id UUID,
    reserved_at TIMESTAMPTZ,
    received_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    processed_at TIMESTAMPTZ,
    extra_info JSONB
);
CREATE INDEX idx_event_entries_status_scheduled_at ON event_entries (status, scheduled_at);
CREATE INDEX idx_event_entries_reservation_id ON event_entries (reservation_id);

CREATE TABLE outbox_entries (
    id UUID PRIMARY KEY,
    status INT NOT NULL DEFAULT 0,
    payload TEXT NOT NULL,
    meta JSONB NOT NULL,
    scheduled_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    attempts INT NOT NULL DEFAULT 0,
    reservation_id UUID,
    reserved_at TIMESTAMPTZ,
    received_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    processed_at TIMESTAMPTZ,
    last_error TEXT,
    extra_info JSONB
);
CREATE INDEX idx_outbox_entries_status_scheduled_at ON outbox_entries (status, scheduled_at);
CREATE INDEX idx_outbox_entries_status_reserved_at ON outbox_entries (status, reserved_at);

-- Transactional index creation blocks writes; schedule a maintenance window.
CREATE INDEX idx_inbox_entries_scheduled_at ON inbox_entries USING btree (scheduled_at);
CREATE INDEX idx_command_entries_scheduled_at ON command_entries USING btree (scheduled_at);
CREATE INDEX idx_event_entries_scheduled_at ON event_entries USING btree (scheduled_at);
CREATE INDEX idx_outbox_entries_scheduled_at ON outbox_entries USING btree (scheduled_at);

CREATE TABLE event_store (
    id BIGSERIAL PRIMARY KEY,
    agg_id VARCHAR(255) NOT NULL,
    sequence BIGINT NOT NULL CHECK (sequence > 0),
    payload_type VARCHAR(255) NOT NULL,
    payload JSONB NOT NULL,
    metadata JSONB NOT NULL,
    recorded_on TIMESTAMPTZ NOT NULL,
    UNIQUE (agg_id, sequence)
);

