//! `poentaro public`: what anyone calls without signing in. It runs the
//! same code as the REST routes it replaces.

use poentaro_protocol::public::{ClaimOriginHandoffError, MigrationId, UploadOriginHandoffError};
use poentaro_protocol::public::{
    FolderFile, OpenSharedLinkError, Shared, SharedFile, SharedFolder,
};
use poentaro_protocol::public::{Session, SignInPasswordError, SignUpPasswordError};
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
            }) => Ok(Err(SignInPasswordError::LockedFor {
                duration: std::time::Duration::from_secs(retry_after_seconds.unsigned_abs()),
            })),
            Err(ApiError::InvalidEmail) => Ok(Err(SignInPasswordError::InvalidEmail)),
            Err(ApiError::RateLimited) => Ok(Err(SignInPasswordError::TooManyRequests)),
            Err(error) => Err(internal(error)),
        }
    }

    async fn open_shared_link(&self, share_id: String) -> Outcome<Shared, OpenSharedLinkError> {
        match shares::read_share(self.d1, &share_id).await {
            Ok(doc) => Ok(Ok(shared(doc))),
            Err(ApiError::SharedFolderTooLarge { message }) => {
                Ok(Err(OpenSharedLinkError::TooLarge { message }))
            }
            Err(error) => Err(internal(error)),
        }
    }

    async fn upload_origin_handoff(
        &self,
        payload: String,
    ) -> Outcome<MigrationId, UploadOriginHandoffError> {
        match origin_migrations::store(self.env, self.d1, &payload, self.meta).await {
            Ok(id) => Ok(Ok(id.into())),
            Err(ApiError::OriginMigrationTooLarge { message }) => {
                Ok(Err(UploadOriginHandoffError::TooLarge { message }))
            }
            Err(ApiError::RateLimited) => Ok(Err(UploadOriginHandoffError::TooManyRequests)),
            Err(error) => Err(internal(error)),
        }
    }

    async fn claim_origin_handoff(
        &self,
        id: MigrationId,
    ) -> Outcome<String, ClaimOriginHandoffError> {
        match origin_migrations::take(self.d1, &id.0).await {
            Ok(payload) => Ok(Ok(payload)),
            Err(ApiError::NotFound) => Ok(Err(ClaimOriginHandoffError::Missing)),
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
