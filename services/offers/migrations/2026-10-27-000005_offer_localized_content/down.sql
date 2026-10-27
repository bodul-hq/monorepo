-- Additional language buckets require a backup or event replay after downgrading.
ALTER TABLE read_offers DROP COLUMN content;
