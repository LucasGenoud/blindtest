-- Conversation history for the LLM blindtest assistant. One thread per custom
-- blindtest, so reopening a generated blindtest resumes where it left off.
CREATE TABLE IF NOT EXISTS blindtest_agent_messages (
    id TEXT PRIMARY KEY NOT NULL,
    blindtest_id TEXT NOT NULL REFERENCES custom_blindtests(id) ON DELETE CASCADE,
    role TEXT NOT NULL CHECK(role IN ('user', 'assistant')),
    content TEXT NOT NULL,
    tracks TEXT NOT NULL DEFAULT '[]',
    created_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_agent_messages_blindtest ON blindtest_agent_messages(blindtest_id, created_at);
