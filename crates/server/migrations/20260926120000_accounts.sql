-- Accounts and sessions (H5): the identity of an account, its sessions, and the tokens of the
-- links it is emailed. Tokens are stored as their SHA-256, never as themselves.

alter table accounts
  add column email citext not null unique,
  add column password_hash text not null,
  add column email_verified_at timestamptz,
  add column language text not null default 'en',
  add column status text not null default 'active' check (status in ('active', 'suspended'));

create table sessions (
  token_hash bytea primary key,         -- SHA-256 of the cookie's token
  account_id uuid not null references accounts (id) on delete cascade,
  created_at timestamptz not null,
  last_seen_at timestamptz not null,
  expires_at timestamptz not null
);
create index sessions_account on sessions (account_id);
create index sessions_expires on sessions (expires_at);

create table email_tokens (
  token_hash bytea primary key,
  account_id uuid not null references accounts (id) on delete cascade,
  purpose text not null check (purpose in ('verify_email', 'reset_password')),
  created_at timestamptz not null,
  expires_at timestamptz not null,
  used_at timestamptz
);
create index email_tokens_account on email_tokens (account_id);
create index email_tokens_expires on email_tokens (expires_at);
