-- Transactional index creation blocks writes; schedule a maintenance window.
CREATE INDEX idx_inbox_entries_scheduled_at ON inbox_entries USING btree (scheduled_at);
CREATE INDEX idx_command_entries_scheduled_at ON command_entries USING btree (scheduled_at);
CREATE INDEX idx_event_entries_scheduled_at ON event_entries USING btree (scheduled_at);
CREATE INDEX idx_outbox_entries_scheduled_at ON outbox_entries USING btree (scheduled_at);
