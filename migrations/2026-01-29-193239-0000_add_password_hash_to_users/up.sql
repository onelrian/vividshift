-- Add password_hash column to users table
ALTER TABLE users ADD COLUMN password_hash VARCHAR(255);

-- Make it NOT NULL after we've created default admin
-- We'll update this in the backend code to handle the transition
