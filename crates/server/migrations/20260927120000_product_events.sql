create table product_events (
  id bigint generated always as identity primary key,
  name text not null,
  subject text,                    -- keyed hash of the account, or null
  properties jsonb not null default '{}',
  platform text,
  app_version text,
  language text,
  occurred_at timestamptz not null
);
create index product_events_name_time on product_events (name, occurred_at);
create index product_events_subject_time on product_events (subject, occurred_at);
