-- Admin only: never loaded by the worker. Read-only preview of what
-- migration `0011_poentaro_file_extension.sql` renames (or renamed: run
-- after the deploy, it lists the files still on the legacy extension, which
-- should only be ones whose new name was taken). Run from
-- `crates/live-share-worker`:
--
--   pnpm exec wrangler d1 execute jianpu-live-share --remote \
--     --command="$(cat queries/admin/preview_poentaro_file_extension.sql)"
--
-- The migration itself needs no manual step: the worker's deploy workflow
-- (`.github/workflows/live-share-worker.yml`) applies it to production.

SELECT
    files.owner_user_id,
    files.name,
    substr(files.name, 1, length(files.name) - length('.jianpu'))
    || '.poentaro' AS new_name,
    files.trashed_at IS NOT NULL AS binned,
    taken.name IS NOT NULL AS keeps_legacy_name
FROM files
LEFT JOIN files AS taken
    ON
        files.owner_user_id = taken.owner_user_id
        AND taken.name
        = substr(files.name, 1, length(files.name) - length('.jianpu'))
        || '.poentaro'
WHERE substr(files.name, -length('.jianpu')) = '.jianpu'
ORDER BY files.owner_user_id, files.name;
