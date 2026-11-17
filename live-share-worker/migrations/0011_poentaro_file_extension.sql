-- The app was renamed from jianpu-generator to Poentaro, and the extension
-- of its files from `.jianpu` to `.poentaro` (see "Rename to Poentaro" in
-- ARCHITECTURE.md). Renames every file, live or binned, whose name ends in
-- `.jianpu` (case-sensitive, like the app). A file whose new name its owner
-- already uses keeps its legacy name, which the app still opens, so the
-- idx_files_owner_name unique index can never fail here. `updated_at` is
-- left alone: nobody edited the file.

UPDATE files
SET name = substr(name, 1, length(name) - length('.jianpu')) || '.poentaro'
WHERE
    substr(name, -length('.jianpu')) = '.jianpu'
    AND NOT EXISTS (
        SELECT 1
        FROM files AS taken
        WHERE
            taken.owner_user_id = files.owner_user_id
            AND taken.name
            = substr(files.name, 1, length(files.name) - length('.jianpu'))
            || '.poentaro'
    );
