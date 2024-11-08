-- Add migration script here
-- create user table
CREATE TABLE IF NOT EXISTS users (
    id BIGSERIAL PRIMARY KEY,
    fullname VARCHAR(64) NOT NULL,
    email VARCHAR(64) NOT NULL,
    -- hashed argon2 password
    password_hash VARCHAR(97) NOT NULL,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

-- create chat type: single, group, private_channel, public_channel
CREATE TYPE chat_type AS ENUM ('single', 'group', 'private_channel', 'public_channel');

-- create chat table
CREATE TABLE IF NOT EXISTS chats (
    id BIGSERIAL PRIMARY KEY,
    name VARCHAR(32) NOT NULL,
    type chat_type NOT NULL,
    -- user id list for group chat
    members BIGINT[] NOT NULL,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

-- create message table
CREATE TABLE IF NOT EXISTS messages (
    id BIGSERIAL PRIMARY KEY,
    chat_id BIGINT NOT NULL REFERENCES chats(id),
    sender_id BIGINT NOT NULL REFERENCES users(id),
    content TEXT NOT NULL,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

-- create index for messages for chat_id and created_at order by created_at desc
CREATE INDEX IF NOT EXISTS chat_id_created_at_nidex ON messages(chat_id, created_at DESC);

-- create index for messages for sender_id
CREATE INDEX IF NOT EXISTS sender_id_index ON messages(sender_id);
