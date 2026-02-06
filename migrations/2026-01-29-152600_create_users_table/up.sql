-- Up.sql
CREATE TABLE IF NOT EXISTS users (
    id TEXT PRIMARY KEY, -- Supabase UUID
    username TEXT NOT NULL UNIQUE,
    email TEXT NOT NULL UNIQUE,
    role TEXT NOT NULL DEFAULT 'USER' CHECK (role IN ('ADMIN', 'USER'))
);

-- Seed with a placeholder for the initial admin. 
-- Note: The actual Supabase UID will be linked upon first login or manual update.
-- But we can pre-specify the admin@admin.com email as the seed target.
INSERT INTO users (id, username, email, role) 
VALUES ('placeholder-admin-id', 'admin', 'admin@admin.com', 'ADMIN')
ON CONFLICT (email) DO UPDATE SET role = 'ADMIN';
