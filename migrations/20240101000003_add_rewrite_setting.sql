-- Add rewrite setting to users table
ALTER TABLE users ADD COLUMN rewrite_enabled BOOLEAN NOT NULL DEFAULT TRUE; 