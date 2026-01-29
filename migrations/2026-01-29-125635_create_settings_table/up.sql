CREATE TABLE IF NOT EXISTS settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL
);

INSERT INTO settings (key, value) VALUES ('assignment_interval_days', '14') ON CONFLICT (key) DO NOTHING;
INSERT INTO settings (key, value) VALUES ('discord_enabled', 'false') ON CONFLICT (key) DO NOTHING;
INSERT INTO settings (key, value) VALUES ('discord_webhook_url', '') ON CONFLICT (key) DO NOTHING;
