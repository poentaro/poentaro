//! `poentaro public`: what anyone calls without signing in. It runs the
//! same code as the REST routes it replaces.

use poentaro_protocol::public::account::{Session, SignInPasswordError, SignUpPasswordError};
use poentaro_protocol::public::links::{FolderFile, OpenError, Shared, SharedFile, SharedFolder};
use poentaro_protocol::public::migrations::{ClaimError, UploadError};
use worker::{D1Database, Env};

use super::internal;
use crate::api_error::ApiError;
use crate::app_log::RequestMeta;
use crate::handlers::{auth, origin_migrations, shares};
use crate::protocol::{SessionResponse, SyncedDoc};

type Outcome<T, E> = worker::Result<Result<T, E>>;

pub(super) struct PublicServer<'a> {
    pub(super) d1: &'a D1Database,
    pub(super) env: &'a Env,
    pub(super) meta: &'a RequestMeta,
}

impl poentaro_protocol::public::Server for PublicServer<'_> {
    type Error = worker::Error;
}

fn session(response: SessionResponse) -> Session {
    Session {
        token: response.session_token,
        email: response.email,
    }
}

impl poentaro_protocol::public::account::Server for PublicServer<'_> {
    async fn sign_up_password(
        &self,
        email: String,
        password: String,
    ) -> Outcome<Session, SignUpPasswordError> {
        match auth::create_account(self.env, self.d1, &email, &password, self.meta).await {
            Ok(response) => Ok(Ok(session(response))),
            Err(ApiError::EmailTaken) => Ok(Err(SignUpPasswordError::EmailTaken)),
            Err(ApiError::WeakPassword { message }) => {
                Ok(Err(SignUpPasswordError::WeakPassword { message }))
            }
            Err(ApiError::InvalidEmail) => Ok(Err(SignUpPasswordError::InvalidEmail)),
            Err(ApiError::RateLimited) => Ok(Err(SignUpPasswordError::TooManyRequests)),
            Err(error) => Err(internal(error)),
        }
    }

    async fn sign_in_password(
        &self,
        email: String,
        password: String,
    ) -> Outcome<Session, SignInPasswordError> {
        match auth::start_session(self.env, self.d1, &email, &password, self.meta).await {
            Ok(response) => Ok(Ok(session(response))),
            Err(ApiError::IncorrectCredentials) => Ok(Err(SignInPasswordError::Incorrect)),
            Err(ApiError::SignInLocked {
                retry_after_seconds,
            }) => Ok(Err(SignInPasswordError::LockedForSeconds {
                duration: retry_after_seconds,
            })),
            Err(ApiError::InvalidEmail) => Ok(Err(SignInPasswordError::InvalidEmail)),
            Err(ApiError::RateLimited) => Ok(Err(SignInPasswordError::TooManyRequests)),
            Err(error) => Err(internal(error)),
        }
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

impl poentaro_protocol::public::links::Server for PublicServer<'_> {
    async fn open(&self, share_id: String) -> Outcome<Shared, OpenError> {
        match shares::read_share(self.d1, &share_id).await {
            Ok(doc) => Ok(Ok(shared(doc))),
            Err(ApiError::SharedFolderTooLarge { message }) => {
                Ok(Err(OpenError::TooLarge { message }))
            }
            Err(error) => Err(internal(error)),
        }
    }
}

impl poentaro_protocol::public::migrations::Server for PublicServer<'_> {
    async fn upload(&self, payload: String) -> Outcome<String, UploadError> {
        match origin_migrations::store(self.env, self.d1, &payload, self.meta).await {
            Ok(id) => Ok(Ok(id)),
            Err(ApiError::OriginMigrationTooLarge { message }) => {
                Ok(Err(UploadError::TooLarge { message }))
            }
            Err(ApiError::RateLimited) => Ok(Err(UploadError::TooManyRequests)),
            Err(error) => Err(internal(error)),
        }
    }

    async fn claim(&self, id: String) -> Outcome<String, ClaimError> {
        match origin_migrations::take(self.d1, &id).await {
            Ok(payload) => Ok(Ok(payload)),
            Err(ApiError::NotFound) => Ok(Err(ClaimError::Missing)),
            Err(error) => Err(internal(error)),
        }
    }
}
