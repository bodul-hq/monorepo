-- Mulac's consumer requires a concrete message ID, matching the source event.
UPDATE outbox_entries
SET meta = jsonb_set(meta, '{message_id}', meta->'event_id')
WHERE (meta->'message_id' IS NULL OR meta->'message_id' = 'null'::jsonb)
  AND jsonb_typeof(meta->'event_id') = 'string';
