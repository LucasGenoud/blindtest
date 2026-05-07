ALTER TABLE audios ADD COLUMN s3_object_key TEXT;
ALTER TABLE audios ADD COLUMN processing_status TEXT DEFAULT 'ready';

ALTER TABLE suggestions ADD COLUMN s3_object_key TEXT;
ALTER TABLE suggestions ADD COLUMN processing_status TEXT DEFAULT 'ready';
