//! `files/*`, `files/versions/*` and `folders/*` of `poentaro app`. Paths
//! the server picks (a restore, a folder move) are decided in
//! `crate::naming`.

use poentaro_protocol::app::MoveFolderToError;
use poentaro_protocol::app::{
    CreateFileAtContentError, File, FileSummary, MoveFileToError, ReadFileError, RestoreFileError,
    TrashFileError, WriteFileAtRevisionContentError,
};
use poentaro_protocol::app::{ListVersionsOfFileError, ReadFileAtRevisionError, Version};

use super::{AppServer, Outcome};
use crate::api_error::ApiError;
use crate::db;
use crate::files::FileNameRow;
use crate::folder_share;
use crate::handlers::file_store;
use crate::handlers::protocols::{instant, internal};
use crate::naming::{self, FileSummaryRow, FolderMovePlan};

fn summary(row: FileSummaryRow) -> FileSummary {
    FileSummary {
        id: row.id,
        path: row.name,
        revision: row.revision,
        trashed_at: row.trashed_at.map(instant),
    }
}

impl AppServer<'_> {
    async fn summaries(&self) -> worker::Result<Vec<FileSummaryRow>> {
        db::list_file_summaries(self.d1, &self.user_id).await
    }
}

impl AppServer<'_> {
    pub(super) async fn list_files(&self) -> worker::Result<Vec<FileSummary>> {
        Ok(self.summaries().await?.into_iter().map(summary).collect())
    }

    pub(super) async fn read_file(&self, id: String) -> Outcome<File, ReadFileError> {
        let Some(file) = db::get_file_by_id(self.d1, &id, &self.user_id).await? else {
            return Ok(Err(ReadFileError::Missing));
        };
        Ok(Ok(File {
            summary: FileSummary {
                id: file.id,
                path: file.name,
                revision: file.revision,
                trashed_at: file.trashed_at.map(instant),
            },
            content: file.content,
        }))
    }

    pub(super) async fn write_file_at_revision_content(
        &self,
        id: String,
        revision: i64,
        content: String,
    ) -> Outcome<i64, WriteFileAtRevisionContentError> {
        match file_store::write_content(self.d1, &self.user_id, &id, revision, &content).await {
            Ok(revision) => Ok(Ok(revision)),
            Err(ApiError::NotFound) => Ok(Err(WriteFileAtRevisionContentError::Missing)),
            Err(ApiError::RevisionConflict { .. }) => {
                // Answer with what the file holds now, so the caller can
                // merge without another round trip.
                let current = db::get_file_by_id(self.d1, &id, &self.user_id).await?;
                Ok(Err(match current {
                    None => WriteFileAtRevisionContentError::Missing,
                    Some(file) if file.trashed_at.is_some() => {
                        WriteFileAtRevisionContentError::Trashed
                    }
                    Some(file) => WriteFileAtRevisionContentError::ConflictAtRevisionContent {
                        revision: file.revision,
                        content: file.content,
                    },
                }))
            }
            Err(error) => Err(internal(error)),
        }
    }

    pub(super) async fn create_file_at_content(
        &self,
        id: String,
        path: String,
        content: String,
    ) -> Outcome<FileSummary, CreateFileAtContentError> {
        let created = file_store::create(
            self.d1,
            self.user_id.clone(),
            id.clone(),
            path.clone(),
            content,
        )
        .await;
        match created {
            Ok(file) => Ok(Ok(FileSummary {
                id: file.id,
                path: file.name,
                revision: file.revision,
                trashed_at: file.trashed_at.map(instant),
            })),
            Err(ApiError::NameTaken) => {
                // The unique index doesn't say which key clashed: the path is
                // taken if a file of this account holds it, else the id is.
                let holder = db::get_file_by_name(self.d1, &self.user_id, &path).await?;
                Ok(Err(if holder.is_some_and(|file| file.id != id) {
                    CreateFileAtContentError::Taken
                } else {
                    CreateFileAtContentError::IdTaken
                }))
            }
            Err(ApiError::FileFolderConflict { path }) => {
                Ok(Err(CreateFileAtContentError::WouldBeBothFileAndFolder {
                    path,
                }))
            }
            Err(ApiError::InvalidName { message }) => {
                Ok(Err(CreateFileAtContentError::InvalidPath { message }))
            }
            Err(error) => Err(internal(error)),
        }
    }

    pub(super) async fn move_file_to(
        &self,
        id: String,
        path: String,
    ) -> Outcome<(), MoveFileToError> {
        match file_store::rename(self.d1, &self.user_id, &id, &path).await {
            Ok(()) => Ok(Ok(())),
            Err(ApiError::NotFound) => Ok(Err(MoveFileToError::Missing)),
            Err(ApiError::NameTaken) => Ok(Err(MoveFileToError::Taken)),
            Err(ApiError::FileFolderConflict { path }) => {
                Ok(Err(MoveFileToError::WouldBeBothFileAndFolder { path }))
            }
            Err(ApiError::InvalidName { message }) => {
                Ok(Err(MoveFileToError::InvalidPath { message }))
            }
            Err(error) => Err(internal(error)),
        }
    }

    pub(super) async fn trash_file(&self, id: String) -> Outcome<(), TrashFileError> {
        match file_store::trash(self.d1, &self.user_id, &id).await {
            Ok(()) => Ok(Ok(())),
            Err(ApiError::NotFound) => Ok(Err(TrashFileError::Missing)),
            Err(error) => Err(internal(error)),
        }
    }

    pub(super) async fn restore_file(&self, id: String) -> Outcome<String, RestoreFileError> {
        let files = self.summaries().await?;
        let Some(file) = files.iter().find(|file| file.id == id) else {
            return Ok(Err(RestoreFileError::Missing));
        };
        if file.trashed_at.is_none() {
            return Ok(Err(RestoreFileError::NotTrashed));
        }
        let path = naming::restored_path(&id, &file.name, &files)
            .map_err(|error| worker::Error::RustError(error.to_string()))?;
        match file_store::restore(self.d1, &self.user_id, &id, path.as_str()).await {
            Ok(()) => Ok(Ok(path.as_str().to_string())),
            Err(ApiError::NotFound) => Ok(Err(RestoreFileError::NotTrashed)),
            Err(ApiError::FileFolderConflict { path }) => {
                Ok(Err(RestoreFileError::WouldBeBothFileAndFolder { path }))
            }
            Err(error) => Err(internal(error)),
        }
    }
}

impl AppServer<'_> {
    pub(super) async fn list_versions_of_file(
        &self,
        id: String,
    ) -> Outcome<Vec<Version>, ListVersionsOfFileError> {
        match file_store::versions(self.d1, &self.user_id, &id).await {
            Ok(versions) => Ok(Ok(versions
                .into_iter()
                .map(|version| Version {
                    revision: version.revision,
                    saved_at: instant(version.saved_at),
                })
                .collect())),
            Err(ApiError::NotFound) => Ok(Err(ListVersionsOfFileError::Missing)),
            Err(error) => Err(internal(error)),
        }
    }

    pub(super) async fn read_file_at_revision(
        &self,
        id: String,
        revision: i64,
    ) -> Outcome<String, ReadFileAtRevisionError> {
        if db::get_file_by_id(self.d1, &id, &self.user_id)
            .await?
            .is_none()
        {
            return Ok(Err(ReadFileAtRevisionError::Missing));
        }
        match file_store::version_content(self.d1, &self.user_id, &id, revision).await {
            Ok(content) => Ok(Ok(content)),
            Err(ApiError::NotFound) => Ok(Err(ReadFileAtRevisionError::NoSuchVersion)),
            Err(error) => Err(internal(error)),
        }
    }
}

impl AppServer<'_> {
    pub(super) async fn move_folder_to(
        &self,
        path: String,
        new_path: String,
    ) -> Outcome<Vec<FileSummary>, MoveFolderToError> {
        let validated = folder_share::validate_folder_path(&path)
            .and_then(|from| Ok((from, folder_share::validate_folder_path(&new_path)?)));
        let (from, to) = match validated {
            Ok(paths) => paths,
            Err(ApiError::InvalidName { message }) => {
                return Ok(Err(MoveFolderToError::InvalidPath { message }))
            }
            Err(error) => return Err(internal(error)),
        };
        let live: Vec<FileNameRow> = self
            .summaries()
            .await?
            .into_iter()
            .filter(|file| file.trashed_at.is_none())
            .map(|file| FileNameRow {
                id: file.id,
                name: file.name,
            })
            .collect();
        let moves = match naming::plan_folder_move(&from, &to, &live) {
            FolderMovePlan::Move(moves) => moves,
            FolderMovePlan::AlreadyMoved => Vec::new(),
            FolderMovePlan::NoSuchFolder => return Ok(Err(MoveFolderToError::NoSuchFolder)),
            FolderMovePlan::IntoItself => {
                return Ok(Err(MoveFolderToError::InvalidPath {
                    message: format!("`{new_path}` is inside `{path}`."),
                }))
            }
            FolderMovePlan::DestinationExists => {
                return Ok(Err(MoveFolderToError::TakenByOtherFiles {
                    paths: vec![to.as_str().to_string()],
                }))
            }
        };
        if !moves.is_empty() {
            match file_store::rename_batch(self.d1, &self.user_id, &moves).await {
                Ok(()) => {}
                Err(ApiError::NotFound) => return Ok(Err(MoveFolderToError::NoSuchFolder)),
                Err(ApiError::NamesTaken { names }) => {
                    return Ok(Err(MoveFolderToError::TakenByOtherFiles { paths: names }))
                }
                Err(ApiError::FileFolderConflict { path }) => {
                    return Ok(Err(MoveFolderToError::WouldBeBothFileAndFolder { path }))
                }
                Err(ApiError::InvalidName { message }) => {
                    return Ok(Err(MoveFolderToError::InvalidPath { message }))
                }
                Err(error) => return Err(internal(error)),
            }
        }
        let prefix = format!("{}/", to.as_str());
        Ok(Ok(self
            .summaries()
            .await?
            .into_iter()
            .filter(|file| file.trashed_at.is_none() && file.name.starts_with(&prefix))
            .map(summary)
            .collect()))
    }
}
