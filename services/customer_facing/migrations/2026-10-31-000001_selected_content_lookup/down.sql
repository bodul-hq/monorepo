-- Retain the corrected lookup and indexes: restoring the broken function would
-- silently disable URL uniqueness and prevent the content-storage downgrade.
-- The content-storage down migration removes the function and its indexes.
SELECT 1;
