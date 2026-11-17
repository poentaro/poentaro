//! `poentaro app`: what the signed-in web app calls. The session token was
//! checked before any method runs, so every method acts for `user_id`. It
//! runs the same cores as the REST routes it replaces.

mod files;
mod links;
mod server;

use poentaro_protocol::app::ChangePasswordFromToError;
use poentaro_protocol::app::{CreatedKey, Key, KeyId, RevokeAgentKeyError};
use worker::D1Database;

use super::{instant, internal};
use crate::api_error::ApiError;
use crate::api_keys::{ApiKeySummary, CreatedApiKey};
use crate::app_log::RequestMeta;
use crate::db;
use crate::handlers::auth::{self, PasswordChange, SignedInSession};
use crate::handlers::{api_keys, now_millis};

type Outcome<T, E> = worker::Result<Result<T, E>>;

pub(super) struct AppServer<'a> {
    pub(super) d1: &'a D1Database,
    pub(super) meta: &'a RequestMeta,
    pub(super) token: String,
    pub(super) user_id: String,
}

impl AppServer<'_> {
    pub(super) async fn sign_out(&self) -> worker::Result<()> {
        auth::end_session(self.d1, &self.token, self.meta)
            .await
            .map_err(internal)
    }

    pub(super) async fn change_password_from_to(
        &self,
        current_password: String,
        new_password: String,
    ) -> Outcome<(), ChangePasswordFromToError> {
        let session = SignedInSession {
            token: &self.token,
            user_id: &self.user_id,
        };
        let passwords = PasswordChange {
            current: &current_password,
            new: &new_password,
        };
        match auth::replace_password(self.d1, session, passwords, self.meta).await {
            Ok(()) => Ok(Ok(())),
            Err(ApiError::IncorrectCredentials) => {
                Ok(Err(ChangePasswordFromToError::IncorrectPassword))
            }
            Err(ApiError::SignInLocked {
                retry_after_seconds,
            }) => Ok(Err(ChangePasswordFromToError::LockedFor {
                duration: std::time::Duration::from_secs(retry_after_seconds.unsigned_abs()),
            })),
            Err(ApiError::WeakPassword { message }) => {
                Ok(Err(ChangePasswordFromToError::WeakPassword { message }))
            }
            Err(error) => Err(internal(error)),
        }
    }
}

fn key(summary: ApiKeySummary) -> Key {
    Key {
        id: summary.id.into(),
        prefix: summary.prefix,
        created_at: instant(summary.created_at),
        last_used_at: summary.last_used_at.map(instant),
        expires_at: instant(summary.expires_at),
        expired: summary.expired,
    }
}

impl AppServer<'_> {
    pub(super) async fn list_agent_keys(&self) -> worker::Result<Vec<Key>> {
        let now = now_millis();
        Ok(db::list_api_keys(self.d1, &self.user_id)
            .await?
            .into_iter()
            .map(|stored| key(ApiKeySummary::at(stored, now)))
            .collect())
    }

    pub(super) async fn create_agent_key(&self) -> worker::Result<CreatedKey> {
        let CreatedApiKey {
            key: summary,
            prompt,
        } = api_keys::create_key(self.d1, &self.user_id, self.meta)
            .await
            .map_err(internal)?;
        Ok(CreatedKey {
            key: key(summary),
            prompt,
        })
    }

    pub(super) async fn revoke_agent_key(&self, id: KeyId) -> Outcome<(), RevokeAgentKeyError> {
        match api_keys::revoke_key(self.d1, &self.user_id, &id.0, self.meta).await {
            Ok(()) => Ok(Ok(())),
            Err(ApiError::NotFound) => Ok(Err(RevokeAgentKeyError::MissingKey)),
            Err(error) => Err(internal(error)),
        }
    }
}
