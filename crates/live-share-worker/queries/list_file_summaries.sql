-- list_file_summaries
-- Every file of owner ?1, in the trash or not, without its content: what
-- `files/list` returns, and the paths `files/restore` and `folders/move`
-- name around.
SELECT id,
       name,
       revision,
       trashed_at
FROM files
WHERE owner_user_id = ?1;
