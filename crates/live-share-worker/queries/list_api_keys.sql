-- An account's API keys, newest first.
SELECT id,
       prefix,
       created_at,
       last_used_at
FROM api_keys
WHERE user_id = ?1
ORDER BY created_at DESC;
