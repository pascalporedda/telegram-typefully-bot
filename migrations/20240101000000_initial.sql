-- Create users table
CREATE TABLE IF NOT EXISTS users (
    telegram_id INTEGER PRIMARY KEY,
    username TEXT NOT NULL,
    typefully_api_key TEXT,
    openai_api_key TEXT,
    created_at DATETIME NOT NULL
);
