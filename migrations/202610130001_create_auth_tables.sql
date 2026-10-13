-- Authentication data; pgcrypto provides bcrypt hashes and CSPRNG tokens.
CREATE EXTENSION IF NOT EXISTS pgcrypto;

CREATE TABLE auth_users (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    name text NOT NULL CHECK (char_length(trim(name)) BETWEEN 2 AND 200),
    email text NOT NULL UNIQUE CHECK (email = lower(email)),
    password_hash text NOT NULL,
    role text NOT NULL CHECK (role IN ('owner', 'admin', 'member')),
    is_active boolean NOT NULL DEFAULT true,
    created_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE auth_sessions (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id uuid NOT NULL REFERENCES auth_users(id) ON DELETE CASCADE,
    token_hash text NOT NULL UNIQUE,
    expires_at timestamptz NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now()
);

CREATE INDEX auth_sessions_user_id_idx ON auth_sessions(user_id);
CREATE INDEX auth_sessions_expires_at_idx ON auth_sessions(expires_at);
-- Initial Owner is deliberately not seeded with a default credential.
