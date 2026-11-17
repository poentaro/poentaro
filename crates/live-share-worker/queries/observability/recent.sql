-- The last 100 `app_log` rows, newest first.
SELECT
    id,
    flow,
    kind,
    outcome,
    user_id,
    subject,
    worker_version,
    origin,
    user_agent,
    detail,
    trace_id,
    DATETIME(at / 1000, 'unixepoch') AS at_utc
FROM app_log
ORDER BY at DESC
LIMIT 100;
