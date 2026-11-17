-- Deletes every unclaimed handoff created before ?1 (ms since epoch). Run by
-- the daily scheduled handler; there are only ever a handful of rows.
DELETE FROM origin_migrations WHERE created_at < ?1;
