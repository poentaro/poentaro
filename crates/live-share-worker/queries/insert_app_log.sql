-- One `app_log` row (see `src/app_log.rs`). Best-effort: the caller swallows
-- a failure rather than failing the request the row belongs to.
INSERT INTO app_log (
    at, flow, kind, outcome, user_id, subject,
    worker_version, origin, user_agent, detail, trace_id
)
VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11);
