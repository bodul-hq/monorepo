-- Selected scalar fields survive; other language buckets require backup/replay.
ALTER TABLE offers DROP COLUMN content;
