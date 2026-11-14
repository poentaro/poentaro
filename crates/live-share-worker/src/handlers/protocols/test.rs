//! `poentaro test`: what end-to-end tests call to set up. Served only when
//! `e2e_reset::EMAILS_VAR` is set.

use poentaro_protocol::test::accounts::{Deleted, ResetError};
use worker::{D1Database, Env};

use crate::db;
use crate::e2e_reset::{emails_to_reset, parse_reset_emails, EMAILS_VAR};

pub(super) struct TestServer<'a> {
    pub(super) d1: &'a D1Database,
    pub(super) env: &'a Env,
}

impl poentaro_protocol::test::Server for TestServer<'_> {
    type Error = worker::Error;
}

impl poentaro_protocol::test::accounts::Server for TestServer<'_> {
    async fn reset(&self, emails: Vec<String>) -> worker::Result<Result<Deleted, ResetError>> {
        let value = self.env.var(EMAILS_VAR)?.to_string();
        let allowed = parse_reset_emails(&value).map_err(|error| {
            worker::Error::RustError(format!(
                "{EMAILS_VAR} is not a JSON array of emails: {error}"
            ))
        })?;
        let requested = (!emails.is_empty()).then_some(emails.as_slice());
        let emails = match emails_to_reset(&allowed, requested) {
            Ok(emails) => emails,
            Err(email) => return Ok(Err(ResetError::IsNotATestAccount { email })),
        };
        let deleted = db::delete_accounts(self.d1, &emails).await?;
        Ok(Ok(Deleted {
            accounts: deleted.accounts as i64,
            shares: deleted.shares as i64,
            files: deleted.files as i64,
        }))
    }
}
