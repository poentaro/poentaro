//! The public endpoints of `api`: what anyone calls without signing in.

use poentaro_protocol::api::{ClaimHandoffError, SizeError, UploadHandoffError};
use poentaro_protocol::api::{CredentialError, RateLimitError};
use poentaro_protocol::api::{FolderFile, Shared, SharedFile, SharedFolder};
use poentaro_protocol::api::{Session, SignInError, SignUpError};
use worker::{D1Database, Env};

use super::internal;
use crate::api_error::ApiError;
use crate::app_log::RequestMeta;
use crate::handlers::{auth, origin_migrations, shares};
use crate::model::{SessionResponse, SyncedDoc};

type Outcome<T, E> = worker::Result<Result<T, E>>;

pub(super) struct PublicServer<'a> {
    pub(super) d1: &'a D1Database,
    pub(super) env: &'a Env,
    pub(super) meta: &'a RequestMeta,
}

impl PublicServer<'_> {
    pub(super) async fn sign_up_password(
        &self,
        email: String,
        password: String,
    ) -> Outcome<Session, SignUpError> {
        match auth::create_account(self.env, self.d1, &email, &password, self.meta).await {
            Ok(response) => Ok(Ok(session(response))),
            Err(ApiError::EmailTaken) => Ok(Err(SignUpError::EmailTaken)),
            Err(ApiError::WeakPassword { message }) => {
                Ok(Err(SignUpError::WeakPassword { message }))
            }
            Err(ApiError::InvalidEmail) => Ok(Err(SignUpError::SignUpRefused {
                error: CredentialError::InvalidEmail,
            })),
            Err(ApiError::RateLimited) => Ok(Err(SignUpError::SignUpRefused {
                error: CredentialError::RateLimited {
                    error: RateLimitError {},
                },
            })),
            Err(error) => Err(internal(error)),
        }
    }

    pub(super) async fn sign_in_password(
        &self,
        email: String,
        password: String,
    ) -> Outcome<Session, SignInError> {
        match auth::start_session(self.env, self.d1, &email, &password, self.meta).await {
            Ok(response) => Ok(Ok(session(response))),
            Err(ApiError::IncorrectCredentials) => Ok(Err(SignInError::Incorrect)),
            Err(ApiError::SignInLocked {
                retry_after_seconds,
            }) => Ok(Err(SignInError::LockedFor {
                duration: std::time::Duration::from_secs(retry_after_seconds.unsigned_abs()),
            })),
            Err(ApiError::InvalidEmail) => Ok(Err(SignInError::SignInRefused {
                error: CredentialError::InvalidEmail,
            })),
            Err(ApiError::RateLimited) => Ok(Err(SignInError::SignInRefused {
                error: CredentialError::RateLimited {
                    error: RateLimitError {},
                },
            })),
            Err(error) => Err(internal(error)),
        }
    }

    pub(super) async fn open_shared_link(&self, share_id: String) -> Outcome<Shared, SizeError> {
        match shares::read_share(self.d1, &share_id).await {
            Ok(doc) => Ok(Ok(shared(doc))),
            Err(ApiError::SharedFolderTooLarge { message }) => Ok(Err(SizeError { message })),
            Err(error) => Err(internal(error)),
        }
    }

    pub(super) async fn upload_origin_handoff(
        &self,
        payload: String,
    ) -> Outcome<String, UploadHandoffError> {
        match origin_migrations::store(self.env, self.d1, &payload, self.meta).await {
            Ok(id) => Ok(Ok(id)),
            Err(ApiError::OriginMigrationTooLarge { message }) => {
                Ok(Err(UploadHandoffError::HandoffTooLarge {
                    error: SizeError { message },
                }))
            }
            Err(ApiError::RateLimited) => Ok(Err(UploadHandoffError::HandoffRateLimited {
                error: RateLimitError {},
            })),
            Err(error) => Err(internal(error)),
        }
    }

    pub(super) async fn claim_origin_handoff(
        &self,
        id: String,
    ) -> Outcome<String, ClaimHandoffError> {
        match origin_migrations::take(self.d1, &id).await {
            Ok(payload) => Ok(Ok(payload)),
            Err(ApiError::NotFound) => Ok(Err(ClaimHandoffError {})),
            Err(error) => Err(internal(error)),
        }
    }
}

fn session(response: SessionResponse) -> Session {
    Session {
        token: response.session_token,
        email: response.email,
    }
}

/// What a link shows, from the REST shape it was read in.
fn shared(doc: SyncedDoc) -> Shared {
    match doc {
        SyncedDoc { ended: true, .. } => Shared::Ended,
        SyncedDoc {
            folder: Some(folder),
            ..
        } => Shared::Folder {
            folder: SharedFolder {
                name: folder.name,
                files: folder
                    .files
                    .into_iter()
                    .map(|file| FolderFile {
                        path: file.path,
                        content: file.content,
                        revision: file.revision,
                    })
                    .collect(),
                empty_folders: folder.empty_folders,
            },
        },
        SyncedDoc {
            filename,
            content,
            revision,
            ..
        } => Shared::File {
            file: SharedFile {
                name: filename,
                content,
                revision,
            },
        },
    }
}
