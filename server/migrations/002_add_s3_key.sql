ALTER TABLE audios ADD COLUMN IF NOT EXISTS s3_object_key TEXT;
ALTER TABLE audios ADD COLUMN IF NOT EXISTS processing_status TEXT DEFAULT 'ready';

ALTER TABLE suggestions ADD COLUMN IF NOT EXISTS s3_object_key TEXT;
ALTER TABLE suggestions ADD COLUMN IF NOT EXISTS processing_status TEXT DEFAULT 'ready';
