-- The hosted library: accounts and their usage, projects, animations. The documents themselves
-- live in object storage, at `document_key`.

create extension if not exists citext;
create extension if not exists pg_trgm;

create table accounts (          -- H5 adds the identity columns
  id uuid primary key,
  plan text not null default 'free',
  storage_used_bytes bigint not null default 0 check (storage_used_bytes >= 0),
  created_at timestamptz not null default now()
);

create table projects (
  id uuid primary key,
  account_id uuid not null references accounts (id) on delete cascade,
  name text not null,
  created_at timestamptz not null,
  updated_at timestamptz not null
);
create index projects_account_updated on projects (account_id, updated_at desc, id desc);

create table animations (
  id uuid primary key,
  account_id uuid not null references accounts (id) on delete cascade,
  project_id uuid not null references projects (id) on delete cascade,
  title text not null,
  width integer not null,
  height integer not null,
  frame_count integer not null,
  document_key text not null unique,
  document_bytes bigint not null,
  version bigint not null,
  created_at timestamptz not null,
  updated_at timestamptz not null
);
create index animations_account_updated on animations (account_id, updated_at desc, id desc);
create index animations_project_updated on animations (project_id, updated_at desc, id desc);
create index animations_title_trgm on animations using gin (title gin_trgm_ops);
