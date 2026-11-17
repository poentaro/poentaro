-- Claims the handoff ?1: deletes it and returns what it held, so a handoff
-- can be read at most once. Expiry is checked by the caller against
-- `created_at`.
DELETE FROM origin_migrations WHERE id = ?1 RETURNING payload, created_at;
