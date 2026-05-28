-- Create configs table (stores credentials and IP for one or more hostnames)
CREATE TABLE IF NOT EXISTS configs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    username TEXT NOT NULL UNIQUE,
    password TEXT NOT NULL,
    current_ip TEXT,
    last_update DATETIME,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Create hosts table (stores individual hostnames that point to a config)
CREATE TABLE IF NOT EXISTS hosts (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    hostname TEXT NOT NULL UNIQUE,
    config_id INTEGER NOT NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (config_id) REFERENCES configs(id) ON DELETE CASCADE
);

CREATE INDEX idx_hostname ON hosts(hostname);
CREATE INDEX idx_config_id ON hosts(config_id);
CREATE INDEX idx_username ON configs(username);
