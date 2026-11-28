//! `api`: the web app's endpoints, generated from the anyx core schema
//! `crates/poentaro-protocol/schemas/api/api.anyx`.
//!
//! The schema makes bearer authentication the default: an endpoint is
//! public only if it says `access (public)`. The generated `Server` trait
//! follows from the types: a signed-in endpoint's method takes the
//! `Account` that `account_from` made from a verified token, and a public
//! one takes none. Nothing here decides who may call what.

use std::cell::Cell;

use poentaro_protocol::api::{
    BadPath, ChangePasswordError, ClaimHandoffError, CreateError, CreatedKey, File, FileError,
    FileSummary, Key, Links, LookupError, MoveFileError, MoveFolderError, Never, ReadVersionError,
    Reply, RestoreError, RevokeKeyError, Session, ShareFolderError, Shared, SignInError,
    SignUpError, SizeError, Status, StopSharingError, StopSharingFolderError, UploadHandoffError,
    Version, WriteError,
};
use worker::{console_error, D1Database, Env};

use super::app::AppServer;
use super::public::PublicServer;
use crate::app_log::RequestMeta;
use crate::identity::resolve_verified_user_id;

pub(super) struct ApiServer<'a> {
    pub(super) d1: &'a D1Database,
    pub(super) env: &'a Env,
    pub(super) meta: &'a RequestMeta,
    /// Whether some call failed inside the server, for the request's log.
    pub(super) failed: Cell<bool>,
}

/// A verified session: what `account_from` makes of a bearer token.
pub(crate) struct SignedIn {
    token: String,
    user_id: String,
}

impl ApiServer<'_> {
    fn public(&self) -> PublicServer<'_> {
        PublicServer {
            d1: self.d1,
            env: self.env,
            meta: self.meta,
        }
    }

    fn app(&self, account: SignedIn) -> AppServer<'_> {
        AppServer {
            d1: self.d1,
            meta: self.meta,
            token: account.token,
            user_id: account.user_id,
        }
    }

    fn reply<T, E>(&self, outcome: worker::Result<Result<T, E>>) -> Reply<T, E> {
        match outcome {
            Ok(Ok(value)) => Reply::Answered { value },
            Ok(Err(problem)) => Reply::Failed { problem },
            Err(error) => {
                console_error!("api: internal error: {error}");
                self.failed.set(true);
                Reply::InternalError
            }
        }
    }

    fn answer<T>(&self, result: worker::Result<T>) -> Reply<T, Never> {
        self.reply(result.map(Ok))
    }
}

impl poentaro_protocol::api::Server for ApiServer<'_> {
    type Account = SignedIn;

    async fn account_from(&self, token: String) -> Option<SignedIn> {
        match resolve_verified_user_id(self.d1, &token, self.meta).await {
            Ok(Ok(user_id)) => Some(SignedIn { token, user_id }),
            Ok(Err(_)) => None,
            Err(error) => {
                console_error!("api: verifying a session: {error}");
                self.failed.set(true);
                None
            }
        }
    }

    async fn sign_up_password(
        &self,
        email: String,
        password: String,
    ) -> Reply<Session, SignUpError> {
        self.reply(self.public().sign_up_password(email, password).await)
    }

    async fn sign_in_password(
        &self,
        email: String,
        password: String,
    ) -> Reply<Session, SignInError> {
        self.reply(self.public().sign_in_password(email, password).await)
    }

    async fn open_shared_link(&self, share_id: String) -> Reply<Shared, SizeError> {
        self.reply(self.public().open_shared_link(share_id).await)
    }

    async fn upload_origin_handoff(&self, payload: String) -> Reply<String, UploadHandoffError> {
        self.reply(self.public().upload_origin_handoff(payload).await)
    }

    async fn claim_origin_handoff(&self, id: String) -> Reply<String, ClaimHandoffError> {
        self.reply(self.public().claim_origin_handoff(id).await)
    }

    async fn sign_out(&self, account: SignedIn) -> Reply<(), Never> {
        self.answer(self.app(account).sign_out().await)
    }

    async fn change_password_from_to(
        &self,
        current_password: String,
        new_password: String,
        account: SignedIn,
    ) -> Reply<(), ChangePasswordError> {
        self.reply(
            self.app(account)
                .change_password_from_to(current_password, new_password)
                .await,
        )
    }

    async fn list_files(&self, account: SignedIn) -> Reply<Vec<FileSummary>, Never> {
        self.answer(self.app(account).list_files().await)
    }

    async fn read_file(&self, id: String, account: SignedIn) -> Reply<File, LookupError> {
        self.reply(self.app(account).read_file(id).await)
    }

    async fn write_file_at_revision_content(
        &self,
        id: String,
        revision: i64,
        content: String,
        account: SignedIn,
    ) -> Reply<i64, WriteError> {
        self.reply(
            self.app(account)
                .write_file_at_revision_content(id, revision, content)
                .await,
        )
    }

    async fn create_file_at_content(
        &self,
        id: String,
        path: String,
        content: String,
        account: SignedIn,
    ) -> Reply<FileSummary, CreateError> {
        self.reply(
            self.app(account)
                .create_file_at_content(id, path, content)
                .await,
        )
    }

    async fn move_file_to(
        &self,
        id: String,
        path: String,
        account: SignedIn,
    ) -> Reply<(), MoveFileError> {
        self.reply(self.app(account).move_file_to(id, path).await)
    }

    async fn trash_file(&self, id: String, account: SignedIn) -> Reply<(), LookupError> {
        self.reply(self.app(account).trash_file(id).await)
    }

    async fn restore_file(&self, id: String, account: SignedIn) -> Reply<String, RestoreError> {
        self.reply(self.app(account).restore_file(id).await)
    }

    async fn list_versions_of_file(
        &self,
        id: String,
        account: SignedIn,
    ) -> Reply<Vec<Version>, LookupError> {
        self.reply(self.app(account).list_versions_of_file(id).await)
    }

    async fn read_file_at_revision(
        &self,
        id: String,
        revision: i64,
        account: SignedIn,
    ) -> Reply<String, ReadVersionError> {
        self.reply(self.app(account).read_file_at_revision(id, revision).await)
    }

    async fn move_folder_to(
        &self,
        path: String,
        new_path: String,
        account: SignedIn,
    ) -> Reply<Vec<FileSummary>, MoveFolderError> {
        self.reply(self.app(account).move_folder_to(path, new_path).await)
    }

    async fn list_agent_keys(&self, account: SignedIn) -> Reply<Vec<Key>, Never> {
        self.answer(self.app(account).list_agent_keys().await)
    }

    async fn create_agent_key(&self, account: SignedIn) -> Reply<CreatedKey, Never> {
        self.answer(self.app(account).create_agent_key().await)
    }

    async fn revoke_agent_key(&self, id: String, account: SignedIn) -> Reply<(), RevokeKeyError> {
        self.reply(self.app(account).revoke_agent_key(id).await)
    }

    async fn start_sharing_file(&self, id: String, account: SignedIn) -> Reply<String, FileError> {
        self.reply(self.app(account).start_sharing_file(id).await)
    }

    async fn stop_sharing_file(
        &self,
        id: String,
        account: SignedIn,
    ) -> Reply<(), StopSharingError> {
        self.reply(self.app(account).stop_sharing_file(id).await)
    }

    async fn sharing_status_of_file(&self, id: String, account: SignedIn) -> Reply<Status, Never> {
        self.answer(self.app(account).sharing_status_of_file(id).await)
    }

    async fn start_sharing_folder(
        &self,
        path: String,
        account: SignedIn,
    ) -> Reply<String, ShareFolderError> {
        self.reply(self.app(account).start_sharing_folder(path).await)
    }

    async fn stop_sharing_folder(
        &self,
        path: String,
        account: SignedIn,
    ) -> Reply<(), StopSharingFolderError> {
        self.reply(self.app(account).stop_sharing_folder(path).await)
    }

    async fn sharing_status_of_folder(
        &self,
        path: String,
        account: SignedIn,
    ) -> Reply<Status, BadPath> {
        self.reply(self.app(account).sharing_status_of_folder(path).await)
    }

    async fn list_shared_links(&self, account: SignedIn) -> Reply<Links, Never> {
        self.answer(self.app(account).list_shared_links().await)
    }
}
