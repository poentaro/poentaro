-- A new API key. Only the key's hash is stored.
INSERT INTO api_keys (id, user_id, token_hash, prefix, created_at)
VALUES (?1, ?2, ?3, ?4, ?5);
