-- Every request gets a trace ID, answered in its `X-Trace-Id` header and in
-- a failure's `ApiErrorBody`. Every row a request writes carries it, and
-- every failed request writes one `request` / `request_failed` row (see
-- `crates/live-share-worker/src/app_log.rs`), so the ID a user sends maps
-- straight to its rows (`queries/observability/by_trace_id.sql`). NULL on
-- rows written before this migration and on rows with no request.

ALTER TABLE app_log ADD COLUMN trace_id TEXT;

CREATE INDEX idx_app_log_trace_id ON app_log(trace_id);
