-- Test-only hard delete for `POST /e2e/reset`: every API key of the users
-- in the JSON array ?1.
DELETE FROM api_keys
WHERE user_id IN (SELECT value FROM json_each(?1));
