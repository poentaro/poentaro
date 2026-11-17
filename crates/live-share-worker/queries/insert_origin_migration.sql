-- Stores a new origin-migration handoff: ?1 id, ?2 payload, ?3 created_at
-- (ms since epoch).
INSERT INTO origin_migrations (id, payload, created_at) VALUES (?1, ?2, ?3);
