-- Add usage tracking
CREATE TABLE IF NOT EXISTS voice_note_usage (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    telegram_id INTEGER NOT NULL,
    duration_seconds INTEGER NOT NULL,
    created_at DATETIME NOT NULL,
    FOREIGN KEY (telegram_id) REFERENCES users(telegram_id)
); 