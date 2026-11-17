-- Which requests are failing? The last 100 failed requests, newest first,
-- with the route, HTTP status and `ApiError` code each was answered with.
-- `outcome = 'error'` rows are 5xx (`internal`: see `message`); the rest
-- are 4xx refusals. `trace_id` is what the user saw.
SELECT
    trace_id,
    JSON_EXTRACT(detail, '$.route') AS route,
    JSON_EXTRACT(detail, '$.status') AS status,
    JSON_EXTRACT(detail, '$.code') AS code,
    JSON_EXTRACT(detail, '$.message') AS message,
    outcome,
    user_id,
    worker_version,
    origin,
    user_agent,
    DATETIME(at / 1000, 'unixepoch') AS at_utc
FROM app_log
WHERE flow = 'request' AND kind = 'request_failed'
ORDER BY at DESC
LIMIT 100;
