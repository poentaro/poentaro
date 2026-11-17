-- What happened to the request a user reported? Every row written under
-- the trace ID shown beside their error, oldest first: the
-- `request_failed` row says which route answered which `code` (and, for
-- `internal`, its `message`); any auth row before it (e.g.
-- `session_rejected`) says why. Replace the ID below with the reported one.
SELECT
    flow,
    kind,
    outcome,
    user_id,
    subject,
    JSON_EXTRACT(detail, '$.route') AS route,
    JSON_EXTRACT(detail, '$.status') AS status,
    JSON_EXTRACT(detail, '$.code') AS code,
    JSON_EXTRACT(detail, '$.message') AS message,
    detail,
    worker_version,
    origin,
    user_agent,
    DATETIME(at / 1000, 'unixepoch') AS at_utc
FROM app_log
WHERE trace_id = '0123456789abcdef'
ORDER BY at, id;
