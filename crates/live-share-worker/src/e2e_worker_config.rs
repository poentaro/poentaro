//! What `web/playwright.config.ts` needs to know about the worker to run it
//! for e2e: the `[vars]` names to set. Written to
//! `web/src/generated/live-share-worker/e2eWorkerConfig.json` by
//! `tests/export_web_config.rs`, so the web side imports them rather than
//! repeating them.

use serde::Serialize;

use crate::e2e_reset::EMAILS_VAR;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct E2eWorkerConfig {
    /// Enables the test-only `POST /test` route; its value is a JSON
    /// array of the emails it may reset.
    pub reset_emails_var: String,
}

pub fn e2e_worker_config() -> E2eWorkerConfig {
    E2eWorkerConfig {
        reset_emails_var: EMAILS_VAR.to_string(),
    }
}
