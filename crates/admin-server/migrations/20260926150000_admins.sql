-- H11: the admin accounts and their sessions, in the admin server's own database. An admin signs
-- in with a password (Argon2id) and a TOTP code; its secret is sealed with LPA_TOTP_KEY.
create extension if not exists citext;

create table admins (
  id uuid primary key,
  email citext not null unique,
  password_hash text not null,
  totp_secret bytea not null,
  totp_last_step bigint not null default 0,
  created_at timestamptz not null,
  disabled_at timestamptz
);

-- A session's token is only kept hashed (SHA-256); it ends 30 minutes after it was last seen,
-- or at expires_at, 8 hours after it began.
create table admin_sessions (
  token_hash bytea primary key,
  admin_id uuid not null references admins (id) on delete cascade,
  created_at timestamptz not null,
  last_seen_at timestamptz not null,
  expires_at timestamptz not null
);

create index admin_sessions_admin on admin_sessions (admin_id);
create index admin_sessions_expires_at on admin_sessions (expires_at);
