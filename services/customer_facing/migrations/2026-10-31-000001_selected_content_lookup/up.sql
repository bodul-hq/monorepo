-- jsonb_array_elements exposes a column named value, shadowing the function
-- argument in the WHERE clause. Use the positional argument for the envelope.
-- Rebuild every expression index after correcting this IMMUTABLE function.
-- The transaction blocks writers; duplicate selected URLs abort it atomically.
LOCK TABLE offers IN ACCESS EXCLUSIVE MODE;
CREATE OR REPLACE FUNCTION offer_selected_content(value jsonb) RETURNS jsonb
LANGUAGE sql IMMUTABLE STRICT PARALLEL SAFE AS $$
 SELECT e FROM pg_catalog.jsonb_array_elements($1->'entries') e
 WHERE (e->'language') = ($1->'selected_language') LIMIT 1
$$;
REINDEX INDEX offers_selected_url_idx;
REINDEX INDEX offers_content_title_search_idx;
REINDEX INDEX offers_content_description_search_idx;
