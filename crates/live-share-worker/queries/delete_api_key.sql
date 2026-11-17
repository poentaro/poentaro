-- Revokes one of the caller's API keys (?2 is the caller).
DELETE FROM api_keys
WHERE id = ?1
  AND user_id = ?2;
