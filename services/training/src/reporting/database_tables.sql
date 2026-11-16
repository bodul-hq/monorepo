-- Statistics only: never scan application rows or trigger ANALYZE here.
SELECT c.relname::text AS table_name,
       COALESCE(s.n_live_tup, 0)::bigint AS row_count,
       pg_table_size(c.oid)::bigint AS table_bytes,
       pg_indexes_size(c.oid)::bigint AS index_bytes
FROM pg_catalog.pg_class c
JOIN pg_catalog.pg_namespace n ON n.oid = c.relnamespace
LEFT JOIN pg_catalog.pg_stat_user_tables s ON s.relid = c.oid
WHERE n.nspname = current_schema()
  AND n.nspname <> 'information_schema'
  AND n.nspname !~ '^pg_'
  AND c.relkind = 'r'
ORDER BY c.relname
