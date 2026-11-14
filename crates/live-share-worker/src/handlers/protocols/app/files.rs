//! `files/*`, `files/versions/*` and `folders/*` of `poentaro app`. Paths
//! the server picks (a restore, a folder move) are decided in
//! `crate::naming`.

use poentaro_protocol::app::files::versions::{ListError, ReadAtRevisionError, Version};
use poentaro_protocol::app::files::{
    CreateAtContentError, File, FileSummary, MoveToError, ReadError, RestoreError, TrashError,
    WriteAtRevisionContentError,
};
use poentaro_protocol::app::folders::MoveToError as FolderMoveToError;

use super::{AppServer, Outcome};
use crate::api_error::ApiError;
use crate::db;
use crate::files::FileNameRow;
use crate::folder_share;
use crate::handlers::file_store;
use crate::handlers::protocols::internal;
use crate::naming::{self, FileSummaryRow, FolderMovePlan};

fn summary(row: FileSummaryRow) -> FileSummary {
    FileSummary {
        id: row.id,
        path: row.name,
        revision: row.revision,
        trashed_at: row.trashed_at,
    }
}

impl AppServer<'_> {
    async fn summaries(&self) -> worker::Result<Vec<FileSummaryRow>> {
        db::list_file_summaries(self.d1, &self.user_id).await
    }
}

impl poentaro_protocol::app::files::Server for AppServer<'_> {
    async fn list(&self) -> worker::Result<Vec<FileSummary>> {
        Ok(self.summaries().await?.into_iter().map(summary).collect())
    }

    async fn read(&self, id: String) -> Outcome<File, ReadError> {
        let Some(file) = db::get_file_by_id(self.d1, &id, &self.user_id).await? else {
            return Ok(Err(ReadError::Missing));
        };
        Ok(Ok(File {
            summary: FileSummary {
                id: file.id,
                path: file.name,
                revision: file.revision,
                trashed_at: file.trashed_at,
            },
            content: file.content,
        }))
    }

    async fn write_at_revision_content(
        &self,
        id: String,
        revision: i64,
        content: String,
    ) -> Outcome<i64, WriteAtRevisionContentError> {
        match file_store::write_content(self.d1, &self.user_id, &id, revision, &content).await {
            Ok(revision) => Ok(Ok(revision)),
            Err(ApiError::NotFound) => Ok(Err(WriteAtRevisionContentError::Missing)),
            Err(ApiError::RevisionConflict { .. }) => {
                // Answer with what the file holds now, so the caller can
                // merge without another round trip.
                let current = db::get_file_by_id(self.d1, &id, &self.user_id).await?;
                Ok(Err(match current {
                    None => WriteAtRevisionContentError::Missing,
                    Some(file) if file.trashed_at.is_some() => WriteAtRevisionContentError::Trashed,
                    Some(file) => WriteAtRevisionContentError::ConflictAtRevisionContent {
                        revision: file.revision,
                        content: file.content,
                    },
                }))
            }
            Err(error) => Err(internal(error)),
        }
    }

    async fn create_at_content(
        &self,
        id: String,
        path: String,
        content: String,
    ) -> Outcome<FileSummary, CreateAtContentError> {
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
                trashed_at: file.trashed_at,
            })),
            Err(ApiError::NameTaken) => {
                // The unique index doesn't say which key clashed: the path is
                // taken if a file of this account holds it, else the id is.
                let holder = db::get_file_by_name(self.d1, &self.user_id, &path).await?;
                Ok(Err(if holder.is_some_and(|file| file.id != id) {
                    CreateAtContentError::Taken
                } else {
                    CreateAtContentError::IdTaken
                }))
            }
            Err(ApiError::FileFolderConflict { path }) => {
                Ok(Err(CreateAtContentError::FileFolderConflict { path }))
            }
            Err(ApiError::InvalidName { message }) => {
                Ok(Err(CreateAtContentError::InvalidPath { message }))
            }
            Err(error) => Err(internal(error)),
        }
    }

    async fn move_to(&self, id: String, path: String) -> Outcome<(), MoveToError> {
        match file_store::rename(self.d1, &self.user_id, &id, &path).await {
            Ok(()) => Ok(Ok(())),
            Err(ApiError::NotFound) => Ok(Err(MoveToError::Missing)),
            Err(ApiError::NameTaken) => Ok(Err(MoveToError::Taken)),
            Err(ApiError::FileFolderConflict { path }) => {
                Ok(Err(MoveToError::FileFolderConflict { path }))
            }
            Err(ApiError::InvalidName { message }) => Ok(Err(MoveToError::InvalidPath { message })),
            Err(error) => Err(internal(error)),
        }
    }

    async fn trash(&self, id: String) -> Outcome<(), TrashError> {
        match file_store::trash(self.d1, &self.user_id, &id).await {
            Ok(()) => Ok(Ok(())),
            Err(ApiError::NotFound) => Ok(Err(TrashError::Missing)),
            Err(error) => Err(internal(error)),
        }
    }

    async fn restore(&self, id: String) -> Outcome<String, RestoreError> {
        let files = self.summaries().await?;
        let Some(file) = files.iter().find(|file| file.id == id) else {
            return Ok(Err(RestoreError::Missing));
        };
        if file.trashed_at.is_none() {
            return Ok(Err(RestoreError::NotTrashed));
        }
        let path = naming::restored_path(&id, &file.name, &files)
            .map_err(|error| worker::Error::RustError(error.to_string()))?;
        match file_store::restore(self.d1, &self.user_id, &id, path.as_str()).await {
            Ok(()) => Ok(Ok(path.as_str().to_string())),
            Err(ApiError::NotFound) => Ok(Err(RestoreError::NotTrashed)),
            Err(ApiError::FileFolderConflict { path }) => {
                Ok(Err(RestoreError::FileFolderConflict { path }))
            }
            Err(error) => Err(internal(error)),
        }
    }
}

impl poentaro_protocol::app::files::versions::Server for AppServer<'_> {
    async fn list(&self, id: String) -> Outcome<Vec<Version>, ListError> {
        match file_store::versions(self.d1, &self.user_id, &id).await {
            Ok(versions) => Ok(Ok(versions
                .into_iter()
                .map(|version| Version {
                    revision: version.revision,
                    saved_at: version.saved_at,
                })
                .collect())),
            Err(ApiError::NotFound) => Ok(Err(ListError::Missing)),
            Err(error) => Err(internal(error)),
        }
    }

    async fn read_at_revision(
        &self,
        id: String,
        revision: i64,
    ) -> Outcome<String, ReadAtRevisionError> {
        if db::get_file_by_id(self.d1, &id, &self.user_id)
            .await?
            .is_none()
        {
            return Ok(Err(ReadAtRevisionError::Missing));
        }
        match file_store::version_content(self.d1, &self.user_id, &id, revision).await {
            Ok(content) => Ok(Ok(content)),
            Err(ApiError::NotFound) => Ok(Err(ReadAtRevisionError::NoSuchVersion)),
            Err(error) => Err(internal(error)),
        }
    }
}

impl poentaro_protocol::app::folders::Server for AppServer<'_> {
    async fn move_to(
        &self,
        path: String,
        new_path: String,
    ) -> Outcome<Vec<FileSummary>, FolderMoveToError> {
        let validated = folder_share::validate_folder_path(&path)
            .and_then(|from| Ok((from, folder_share::validate_folder_path(&new_path)?)));
        let (from, to) = match validated {
            Ok(paths) => paths,
            Err(ApiError::InvalidName { message }) => {
                return Ok(Err(FolderMoveToError::InvalidPath { message }))
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
            FolderMovePlan::NoSuchFolder => return Ok(Err(FolderMoveToError::NoSuchFolder)),
            FolderMovePlan::IntoItself => {
                return Ok(Err(FolderMoveToError::InvalidPath {
                    message: format!("`{new_path}` is inside `{path}`."),
                }))
            }
            FolderMovePlan::DestinationExists => {
                return Ok(Err(FolderMoveToError::DestinationsTaken {
                    paths: vec![to.as_str().to_string()],
                }))
            }
        };
        if !moves.is_empty() {
            match file_store::rename_batch(self.d1, &self.user_id, &moves).await {
                Ok(()) => {}
                Err(ApiError::NotFound) => return Ok(Err(FolderMoveToError::NoSuchFolder)),
                Err(ApiError::NamesTaken { names }) => {
                    return Ok(Err(FolderMoveToError::DestinationsTaken { paths: names }))
                }
                Err(ApiError::FileFolderConflict { path }) => {
                    return Ok(Err(FolderMoveToError::FileFolderConflict { path }))
                }
                Err(ApiError::InvalidName { message }) => {
                    return Ok(Err(FolderMoveToError::InvalidPath { message }))
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
