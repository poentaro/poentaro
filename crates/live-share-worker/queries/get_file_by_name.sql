-- One of the owner's files by its path, active or trashed (names are unique
-- per owner across both, `idx_files_owner_name`). For `POST /agent`, which
-- addresses files by path.
SELECT id,
       owner_user_id,
       name,
       content,
       revision,
       trashed_at,
       created_at,
       updated_at
FROM files
WHERE owner_user_id = ?1
  AND name = ?2;
