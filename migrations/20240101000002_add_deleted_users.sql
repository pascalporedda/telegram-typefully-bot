-- Track deleted users to maintain usage history
CREATE TABLE IF NOT EXISTS deleted_users (
    telegram_id INTEGER PRIMARY KEY,
    total_usage_seconds INTEGER NOT NULL,
    deleted_at DATETIME NOT NULL
); 