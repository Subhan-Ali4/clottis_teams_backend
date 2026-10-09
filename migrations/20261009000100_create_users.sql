-- Simple account foundation for the approved CLOTTIS Teams authentication flow.
-- Passwords are never stored here: only an Argon2id-compatible hash belongs in password_hash.

CREATE EXTENSION IF NOT EXISTS pgcrypto;

CREATE TYPE user_role AS ENUM (
    'owner',
    'admin',
    'mentor',
    'member',
    'viewer'
);

CREATE TYPE user_status AS ENUM (
    'active',
    'inactive'
);

CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    full_name VARCHAR(160) NOT NULL,
    email VARCHAR(320) NOT NULL,
    password_hash TEXT NOT NULL,
    role user_role NOT NULL DEFAULT 'member',
    status user_status NOT NULL DEFAULT 'active',
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT users_email_lowercase_check CHECK (email = LOWER(email))
);

CREATE UNIQUE INDEX users_email_unique_idx ON users (email);

COMMENT ON TABLE users IS 'CLOTTIS Teams accounts. Plaintext passwords, verification state and recovery tokens are intentionally not stored here.';
COMMENT ON COLUMN users.password_hash IS 'Argon2id password hash only; never plaintext.';
