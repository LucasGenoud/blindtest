-- Distinguish flags raised by a person from flags raised automatically by a client
-- when playback fails. Only manual flags take an audio out of rotation.
ALTER TABLE flagged_audios ADD COLUMN auto INTEGER NOT NULL DEFAULT 0;

-- One flag per user per audio. Collapse any pre-existing duplicates first so the
-- index cannot fail to build on an older database.
DELETE FROM flagged_audios WHERE rowid NOT IN (SELECT MIN(rowid) FROM flagged_audios GROUP BY audio_id, user_id);
CREATE UNIQUE INDEX IF NOT EXISTS idx_flagged_audios_unique ON flagged_audios (audio_id, user_id);

-- Password reset was never wired up (no mail is sent), so any token still sitting in
-- the table is a standing account takeover against /changepassword.
UPDATE users SET reset_password_token = NULL WHERE reset_password_token IS NOT NULL;
