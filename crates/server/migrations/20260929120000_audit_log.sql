-- The audit log (H10): every action of an admin through the internal admin API — who, what, on
-- what, why, when, and the state before and after. Append-only: a trigger refuses every update
-- and delete, and a statement trigger every truncate. No entry holds a person's address or a
-- message's text, so that an erasure leaves nothing of them here.

create table audit_log (
  id bigint generated always as identity primary key,
  admin_id text not null,
  admin_email text not null,
  action text not null,              -- user.suspend, support.reply, …
  target_type text not null,
  target_id text not null,
  reason text,
  before jsonb,
  after jsonb,
  at timestamptz not null
);
create index audit_log_admin on audit_log (admin_id, id desc);
create index audit_log_action on audit_log (action, id desc);
create index audit_log_target on audit_log (target_id, id desc);

create function audit_log_append_only() returns trigger language plpgsql as $$
begin raise exception 'audit_log is append-only'; end $$;
create trigger audit_log_append_only before update or delete on audit_log
  for each row execute function audit_log_append_only();
create trigger audit_log_no_truncate before truncate on audit_log
  for each statement execute function audit_log_append_only();
