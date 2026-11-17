-- Resolves an API key's hash to the key and its owner.
SELECT id,
       user_id,
       prefix,
       created_at,
       last_used_at
FROM api_keys
WHERE token_hash = ?1;
