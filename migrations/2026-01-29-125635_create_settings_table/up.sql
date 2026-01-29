CREATE TABLE settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL
);

INSERT INTO settings (key, value) VALUES ('assignment_interval_days', '14');
INSERT INTO settings (key, value) VALUES ('discord_enabled', 'false');
INSERT INTO settings (key, value) VALUES ('discord_webhook_url', '');
