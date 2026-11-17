-- Records when an API key was last used.
UPDATE api_keys
SET last_used_at = ?2
WHERE id = ?1;
