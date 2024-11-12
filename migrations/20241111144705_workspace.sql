-- Add migration script here
-- workspace for users
CREATE TABLE IF NOT EXISTS workspaces (
    id BIGSERIAL PRIMARY KEY,
    name VARCHAR(32) NOT NULL UNIQUE,
    owner_id BIGINT NOT NULL REFERENCES users(id),
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

-- alter users table to add workspace_id
ALTER TABLE users
ADD COLUMN ws_id BIGINT REFERENCES workspaces(id);

-- alter chats table to add ws_id
ALTER TABLE chats
  ADD COLUMN ws_id bigint REFERENCES workspaces(id);

BEGIN;
-- add super user 0
INSERT INTO users (id, fullname, email, password_hash)
VALUES (0, 'admin', 'admin@wrxx.site', '');
-- add workspace for super user 0
INSERT INTO workspaces (id, name, owner_id)
VALUES (0, 'admin', 0);
UPDATE users
SET ws_id = 0
WHERE id = 0;
COMMIT;

-- alter user table to make ws_id not null
ALTER TABLE users
ALTER COLUMN ws_id SET NOT NULL;
