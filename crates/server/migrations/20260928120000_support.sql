-- Support requests (H9): Help → Contact from the app, and the team's answers. The team's internal
-- notes are messages with `internal`, never shown to the person. A request's screenshot lives in
-- object storage, at `screenshot_key`.

create table support_requests (
  id uuid primary key,
  account_id uuid not null references accounts (id) on delete cascade,
  category text not null
    check (category in ('bug', 'account', 'billing', 'data_protection', 'abuse', 'other')),
  status text not null default 'new'
    check (status in ('new', 'in_progress', 'waiting_for_user', 'resolved', 'closed')),
  context jsonb not null,            -- appVersion, platform, language, screen
  screenshot_key text,
  assigned_to text,                  -- an admin id, set through the admin API
  created_at timestamptz not null,
  updated_at timestamptz not null,
  first_response_at timestamptz,
  resolved_at timestamptz
);
create index support_requests_status on support_requests (status, updated_at desc);
create index support_requests_account on support_requests (account_id, updated_at desc);

create table support_messages (
  id uuid primary key,
  request_id uuid not null references support_requests (id) on delete cascade,
  author text not null check (author in ('user', 'team')),
  admin_id text,
  body text not null,
  internal boolean not null default false,
  created_at timestamptz not null
);
create index support_messages_request on support_messages (request_id, created_at);
