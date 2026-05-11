-- Remove rating columns from audios table
-- SQLite does not support DROP COLUMN directly in older versions,
-- but rusqlite/SQLite 3.35+ supports it. Using safe migration approach.
ALTER TABLE audios DROP COLUMN rating;
ALTER TABLE audios DROP COLUMN rating_count;
DROP TABLE IF EXISTS ratings;
