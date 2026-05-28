-- Migration script to convert existing database to new multi-hostname schema
-- Run this with: sqlite3 ddns.db < migrate_db.sql

-- Create configs table
CREATE TABLE IF NOT EXISTS configs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    username TEXT NOT NULL UNIQUE,
    password TEXT NOT NULL,
    current_ip TEXT,
    last_update DATETIME,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Create new hosts table
CREATE TABLE IF NOT EXISTS hosts_new (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    hostname TEXT NOT NULL UNIQUE,
    config_id INTEGER NOT NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (config_id) REFERENCES configs(id) ON DELETE CASCADE
);

-- Migrate data from old hosts table to configs
INSERT INTO configs (username, password, current_ip, last_update, created_at)
SELECT username, password, current_ip, last_update, created_at
FROM hosts;

-- Migrate data from old hosts table to new hosts table
INSERT INTO hosts_new (hostname, config_id, created_at)
SELECT h.hostname, c.id, h.created_at
FROM hosts h
JOIN configs c ON h.username = c.username;

-- Drop old hosts table and rename new one
DROP TABLE hosts;
ALTER TABLE hosts_new RENAME TO hosts;

-- Create indexes
CREATE INDEX IF NOT EXISTS idx_hostname ON hosts(hostname);
CREATE INDEX IF NOT EXISTS idx_config_id ON hosts(config_id);
CREATE INDEX IF NOT EXISTS idx_username ON configs(username);

-- Display migration results
SELECT 'Migration complete!' as status;
SELECT COUNT(*) as total_configs FROM configs;
SELECT COUNT(*) as total_hosts FROM hosts;
