-- Blindtest SQLite Schema

CREATE TABLE IF NOT EXISTS users (
    id TEXT PRIMARY KEY NOT NULL,
    email TEXT UNIQUE NOT NULL,
    name TEXT UNIQUE NOT NULL,
    password TEXT NOT NULL,
    role TEXT NOT NULL DEFAULT 'user' CHECK(role IN ('user', 'contributor', 'administrator')),
    clear_mode INTEGER NOT NULL DEFAULT 0,
    hide_carousel INTEGER NOT NULL DEFAULT 0,
    email_confirmation_token TEXT,
    email_confirmed INTEGER NOT NULL DEFAULT 0,
    reset_password_token TEXT,
    reset_password_expires TEXT,
    register_date TEXT NOT NULL,
    deleted INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS audios (
    id TEXT PRIMARY KEY NOT NULL,
    category TEXT NOT NULL,
    answer TEXT NOT NULL,
    video_url TEXT NOT NULL,
    start_time INTEGER NOT NULL DEFAULT 0,
    superflus INTEGER NOT NULL DEFAULT 0,
    count INTEGER NOT NULL DEFAULT 0,
    submitted_by TEXT NOT NULL REFERENCES users(id),
    added_date TEXT NOT NULL,
    last_updated_by TEXT REFERENCES users(id),
    rating REAL NOT NULL DEFAULT 0.0,
    rating_count INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS flagged_audios (
    id TEXT PRIMARY KEY NOT NULL,
    audio_id TEXT NOT NULL REFERENCES audios(id) ON DELETE CASCADE,
    user_id TEXT NOT NULL REFERENCES users(id),
    report_message TEXT NOT NULL DEFAULT '',
    date TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS custom_blindtests (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL,
    public INTEGER NOT NULL DEFAULT 0,
    owner_id TEXT NOT NULL REFERENCES users(id),
    added_date TEXT NOT NULL,
    blindtest_list TEXT NOT NULL DEFAULT '[]'
);

CREATE TABLE IF NOT EXISTS ratings (
    id TEXT PRIMARY KEY NOT NULL,
    audio_id TEXT NOT NULL REFERENCES audios(id) ON DELETE CASCADE,
    user_id TEXT NOT NULL REFERENCES users(id),
    rating REAL NOT NULL,
    added_date TEXT NOT NULL,
    UNIQUE(audio_id, user_id)
);

CREATE TABLE IF NOT EXISTS suggestions (
    id TEXT PRIMARY KEY NOT NULL,
    category TEXT NOT NULL,
    answer TEXT NOT NULL,
    video_url TEXT NOT NULL,
    start_time INTEGER NOT NULL DEFAULT 0,
    superflus INTEGER NOT NULL DEFAULT 0,
    submitted_by TEXT NOT NULL REFERENCES users(id),
    added_date TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS chat_messages (
    id TEXT PRIMARY KEY NOT NULL,
    user_id TEXT NOT NULL REFERENCES users(id),
    username TEXT NOT NULL,
    message_value TEXT NOT NULL,
    date TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS canvas_pixels (
    x INTEGER NOT NULL,
    y INTEGER NOT NULL,
    color TEXT NOT NULL DEFAULT 'ffffff',
    user_id TEXT REFERENCES users(id),
    updated_at TEXT,
    PRIMARY KEY (x, y)
);

CREATE TABLE IF NOT EXISTS stats (
    id TEXT PRIMARY KEY NOT NULL,
    category TEXT NOT NULL,
    user_id TEXT REFERENCES users(id),
    date TEXT NOT NULL,
    metadata TEXT NOT NULL DEFAULT '{}'
);

-- Indexes for performance
CREATE INDEX IF NOT EXISTS idx_audios_category ON audios(category);
CREATE INDEX IF NOT EXISTS idx_audios_count ON audios(count);
CREATE INDEX IF NOT EXISTS idx_audios_submitted_by ON audios(submitted_by);
CREATE INDEX IF NOT EXISTS idx_flagged_audios_audio_id ON flagged_audios(audio_id);
CREATE INDEX IF NOT EXISTS idx_stats_category ON stats(category);
CREATE INDEX IF NOT EXISTS idx_stats_date ON stats(date);
CREATE INDEX IF NOT EXISTS idx_chat_messages_date ON chat_messages(date);
CREATE INDEX IF NOT EXISTS idx_custom_blindtests_owner ON custom_blindtests(owner_id);
CREATE INDEX IF NOT EXISTS idx_ratings_audio ON ratings(audio_id);
