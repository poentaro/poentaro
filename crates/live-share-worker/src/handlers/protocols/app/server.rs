//! The generated flat protocol, delegated to the account, file and link handlers.

use super::{AppServer, Outcome};
use poentaro_protocol::app::{
    ChangePasswordFromToError, CreateFileAtContentError, CreatedKey, File, FileSummary, Key, KeyId,
    Links, ListVersionsOfFileError, MoveFileToError, MoveFolderToError, ReadFileAtRevisionError,
    ReadFileError, RestoreFileError, RevokeAgentKeyError, SharingStatusOfFolderError,
    StartSharingFileError, StartSharingFolderError, Status, StopSharingFileError,
    StopSharingFolderError, TrashFileError, Version, WriteFileAtRevisionContentError,
};

impl poentaro_protocol::app::Server for AppServer<'_> {
    type Error = worker::Error;

    async fn move_folder_to(
        &self,
        path: String,
        new_path: String,
    ) -> Outcome<Vec<FileSummary>, MoveFolderToError> {
        self.move_folder_to(path, new_path).await
    }
    async fn list_versions_of_file(
        &self,
        id: String,
    ) -> Outcome<Vec<Version>, ListVersionsOfFileError> {
        self.list_versions_of_file(id).await
    }
    async fn read_file_at_revision(
        &self,
        id: String,
        revision: i64,
    ) -> Outcome<String, ReadFileAtRevisionError> {
        self.read_file_at_revision(id, revision).await
    }
    async fn list_files(&self) -> worker::Result<Vec<FileSummary>> {
        self.list_files().await
    }
    async fn read_file(&self, id: String) -> Outcome<File, ReadFileError> {
        self.read_file(id).await
    }
    async fn write_file_at_revision_content(
        &self,
        id: String,
        revision: i64,
        content: String,
    ) -> Outcome<i64, WriteFileAtRevisionContentError> {
        self.write_file_at_revision_content(id, revision, content)
            .await
    }
    async fn create_file_at_content(
        &self,
        id: String,
        path: String,
        content: String,
    ) -> Outcome<FileSummary, CreateFileAtContentError> {
        self.create_file_at_content(id, path, content).await
    }
    async fn move_file_to(&self, id: String, path: String) -> Outcome<(), MoveFileToError> {
        self.move_file_to(id, path).await
    }
    async fn trash_file(&self, id: String) -> Outcome<(), TrashFileError> {
        self.trash_file(id).await
    }
    async fn restore_file(&self, id: String) -> Outcome<String, RestoreFileError> {
        self.restore_file(id).await
    }
    async fn start_sharing_file(&self, id: String) -> Outcome<String, StartSharingFileError> {
        self.start_sharing_file(id).await
    }
    async fn stop_sharing_file(&self, id: String) -> Outcome<(), StopSharingFileError> {
        self.stop_sharing_file(id).await
    }
    async fn sharing_status_of_file(&self, id: String) -> worker::Result<Status> {
        self.sharing_status_of_file(id).await
    }
    async fn start_sharing_folder(&self, path: String) -> Outcome<String, StartSharingFolderError> {
        self.start_sharing_folder(path).await
    }
    async fn stop_sharing_folder(&self, path: String) -> Outcome<(), StopSharingFolderError> {
        self.stop_sharing_folder(path).await
    }
    async fn sharing_status_of_folder(
        &self,
        path: String,
    ) -> Outcome<Status, SharingStatusOfFolderError> {
        self.sharing_status_of_folder(path).await
    }
    async fn list_shared_links(&self) -> worker::Result<Links> {
        self.list_shared_links().await
    }
    async fn list_agent_keys(&self) -> worker::Result<Vec<Key>> {
        self.list_agent_keys().await
    }
    async fn create_agent_key(&self) -> worker::Result<CreatedKey> {
        self.create_agent_key().await
    }
    async fn revoke_agent_key(&self, id: KeyId) -> Outcome<(), RevokeAgentKeyError> {
        self.revoke_agent_key(id).await
    }
    async fn sign_out(&self) -> worker::Result<()> {
        self.sign_out().await
    }
    async fn change_password_from_to(
        &self,
        current_password: String,
        new_password: String,
    ) -> Outcome<(), ChangePasswordFromToError> {
        self.change_password_from_to(current_password, new_password)
            .await
    }
}
