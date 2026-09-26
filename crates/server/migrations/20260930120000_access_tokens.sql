-- Personal access tokens (A3): what an agent sends to the hosted MCP endpoint, kept by the SHA-256
-- of their secret; and the MCP calls of each account per UTC day, for the plan's daily ceiling.

create table access_tokens (
  id uuid primary key,
  account_id uuid not null references accounts (id) on delete cascade,
  name text not null,
  token_hash bytea not null unique,     -- SHA-256
  prefix text not null,                 -- "lp_pat_" and 4 characters, for display
  scopes text[] not null,
  created_at timestamptz not null,
  expires_at timestamptz not null,
  last_used_at timestamptz,             -- updated at most once a minute
  revoked_at timestamptz
);
create index access_tokens_account on access_tokens (account_id, created_at desc);

create table mcp_usage (
  account_id uuid not null references accounts (id) on delete cascade,
  day date not null,
  calls integer not null,
  primary key (account_id, day)
);
