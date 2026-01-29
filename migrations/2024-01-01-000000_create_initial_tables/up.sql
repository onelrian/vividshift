-- Up.sql
CREATE TABLE IF NOT EXISTS people (
    id SERIAL PRIMARY KEY,
    name TEXT NOT NULL,
    group_type TEXT NOT NULL CHECK (group_type IN ('A', 'B')),
    active BOOLEAN NOT NULL DEFAULT TRUE
);

CREATE TABLE IF NOT EXISTS assignments (
    id SERIAL PRIMARY KEY,
    person_id INTEGER NOT NULL REFERENCES people(id),
    task_name TEXT NOT NULL,
    assigned_at TIMESTAMP NOT NULL DEFAULT NOW()
);
