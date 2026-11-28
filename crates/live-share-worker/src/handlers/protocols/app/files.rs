//! The file, version and folder endpoints of `api`. Paths the server picks
//! (a restore, a folder move) are decided in `crate::naming`.

use poentaro_protocol::api::{BadPath, ConflictError, FileError, FolderLookupError, PathError};
use poentaro_protocol::api::{CreateError, File, FileSummary, LookupError, MoveFileError};
use poentaro_protocol::api::{
    MoveFolderError, ReadVersionError, RestoreError, Version, WriteError,
};

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
    pub(in crate::handlers::protocols) async fn list_files(
        &self,
    ) -> worker::Result<Vec<FileSummary>> {
        Ok(self.summaries().await?.into_iter().map(summary).collect())
    }

    pub(in crate::handlers::protocols) async fn read_file(
        &self,
        id: String,
    ) -> Outcome<File, LookupError> {
        let Some(file) = db::get_file_by_id(self.d1, &id, &self.user_id).await? else {
            return Ok(Err(LookupError {}));
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

    pub(in crate::handlers::protocols) async fn write_file_at_revision_content(
        &self,
        id: String,
        revision: i64,
        content: String,
    ) -> Outcome<i64, WriteError> {
        match file_store::write_content(self.d1, &self.user_id, &id, revision, &content).await {
            Ok(revision) => Ok(Ok(revision)),
            Err(ApiError::NotFound) => Ok(Err(WriteError::WriteRefused {
                error: FileError::NotFound {
                    error: LookupError {},
                },
            })),
            Err(ApiError::RevisionConflict { .. }) => {
                // Answer with what the file holds now, so the caller can
                // merge without another round trip.
                let current = db::get_file_by_id(self.d1, &id, &self.user_id).await?;
                Ok(Err(match current {
                    None => WriteError::WriteRefused {
                        error: FileError::NotFound {
                            error: LookupError {},
                        },
                    },
                    Some(file) if file.trashed_at.is_some() => WriteError::WriteRefused {
                        error: FileError::Trashed,
                    },
                    Some(file) => WriteError::ConflictAtRevisionContent {
                        revision: file.revision,
                        content: file.content,
                    },
                }))
            }
            Err(error) => Err(internal(error)),
        }
    }

    pub(in crate::handlers::protocols) async fn create_file_at_content(
        &self,
        id: String,
        path: String,
        content: String,
    ) -> Outcome<FileSummary, CreateError> {
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
                    CreateError::CreateRefused {
                        error: PathError::Taken,
                    }
                } else {
                    CreateError::IdTaken
                }))
            }
            Err(ApiError::FileFolderConflict { path }) => Ok(Err(CreateError::CreateRefused {
                error: PathError::PathConflict {
                    error: ConflictError { path },
                },
            })),
            Err(ApiError::InvalidName { message }) => Ok(Err(CreateError::CreateRefused {
                error: PathError::BadPath {
                    error: BadPath { message },
                },
            })),
            Err(error) => Err(internal(error)),
        }
    }

    pub(in crate::handlers::protocols) async fn move_file_to(
        &self,
        id: String,
        path: String,
    ) -> Outcome<(), MoveFileError> {
        match file_store::rename(self.d1, &self.user_id, &id, &path).await {
            Ok(()) => Ok(Ok(())),
            Err(ApiError::NotFound) => Ok(Err(MoveFileError::MoveNotFound {
                error: LookupError {},
            })),
            Err(ApiError::NameTaken) => Ok(Err(MoveFileError::MoveRefused {
                error: PathError::Taken,
            })),
            Err(ApiError::FileFolderConflict { path }) => Ok(Err(MoveFileError::MoveRefused {
                error: PathError::PathConflict {
                    error: ConflictError { path },
                },
            })),
            Err(ApiError::InvalidName { message }) => Ok(Err(MoveFileError::MoveRefused {
                error: PathError::BadPath {
                    error: BadPath { message },
                },
            })),
            Err(error) => Err(internal(error)),
        }
    }

    pub(in crate::handlers::protocols) async fn trash_file(
        &self,
        id: String,
    ) -> Outcome<(), LookupError> {
        match file_store::trash(self.d1, &self.user_id, &id).await {
            Ok(()) => Ok(Ok(())),
            Err(ApiError::NotFound) => Ok(Err(LookupError {})),
            Err(error) => Err(internal(error)),
        }
    }

    pub(in crate::handlers::protocols) async fn restore_file(
        &self,
        id: String,
    ) -> Outcome<String, RestoreError> {
        let files = self.summaries().await?;
        let Some(file) = files.iter().find(|file| file.id == id) else {
            return Ok(Err(RestoreError::RestoreNotFound {
                error: LookupError {},
            }));
        };
        if file.trashed_at.is_none() {
            return Ok(Err(RestoreError::NotTrashed));
        }
        let path = naming::restored_path(&id, &file.name, &files)
            .map_err(|error| worker::Error::RustError(error.to_string()))?;
        match file_store::restore(self.d1, &self.user_id, &id, path.as_str()).await {
            Ok(()) => Ok(Ok(path.as_str().to_string())),
            Err(ApiError::NotFound) => Ok(Err(RestoreError::NotTrashed)),
            Err(ApiError::FileFolderConflict { path }) => Ok(Err(RestoreError::RestoreConflict {
                error: ConflictError { path },
            })),
            Err(error) => Err(internal(error)),
        }
    }
}

impl AppServer<'_> {
    pub(in crate::handlers::protocols) async fn list_versions_of_file(
        &self,
        id: String,
    ) -> Outcome<Vec<Version>, LookupError> {
        match file_store::versions(self.d1, &self.user_id, &id).await {
            Ok(versions) => Ok(Ok(versions
                .into_iter()
                .map(|version| Version {
                    revision: version.revision,
                    saved_at: instant(version.saved_at),
                })
                .collect())),
            Err(ApiError::NotFound) => Ok(Err(LookupError {})),
            Err(error) => Err(internal(error)),
        }
    }

    pub(in crate::handlers::protocols) async fn read_file_at_revision(
        &self,
        id: String,
        revision: i64,
    ) -> Outcome<String, ReadVersionError> {
        if db::get_file_by_id(self.d1, &id, &self.user_id)
            .await?
            .is_none()
        {
            return Ok(Err(ReadVersionError::VersionNotFound {
                error: LookupError {},
            }));
        }
        match file_store::version_content(self.d1, &self.user_id, &id, revision).await {
            Ok(content) => Ok(Ok(content)),
            Err(ApiError::NotFound) => Ok(Err(ReadVersionError::NoSuchVersion)),
            Err(error) => Err(internal(error)),
        }
    }
}

impl AppServer<'_> {
    pub(in crate::handlers::protocols) async fn move_folder_to(
        &self,
        path: String,
        new_path: String,
    ) -> Outcome<Vec<FileSummary>, MoveFolderError> {
        let validated = folder_share::validate_folder_path(&path)
            .and_then(|from| Ok((from, folder_share::validate_folder_path(&new_path)?)));
        let (from, to) = match validated {
            Ok(paths) => paths,
            Err(ApiError::InvalidName { message }) => {
                return Ok(Err(MoveFolderError::BadFolderPath {
                    error: BadPath { message },
                }))
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
            FolderMovePlan::NoSuchFolder => {
                return Ok(Err(MoveFolderError::FolderNotFound {
                    error: FolderLookupError {},
                }))
            }
            FolderMovePlan::IntoItself => {
                return Ok(Err(MoveFolderError::BadFolderPath {
                    error: BadPath {
                        message: format!("`{new_path}` is inside `{path}`."),
                    },
                }))
            }
            FolderMovePlan::DestinationExists => {
                return Ok(Err(MoveFolderError::TakenByOtherFiles {
                    paths: vec![to.as_str().to_string()],
                }))
            }
        };
        if !moves.is_empty() {
            match file_store::rename_batch(self.d1, &self.user_id, &moves).await {
                Ok(()) => {}
                Err(ApiError::NotFound) => {
                    return Ok(Err(MoveFolderError::FolderNotFound {
                        error: FolderLookupError {},
                    }))
                }
                Err(ApiError::NamesTaken { names }) => {
                    return Ok(Err(MoveFolderError::TakenByOtherFiles { paths: names }))
                }
                Err(ApiError::FileFolderConflict { path }) => {
                    return Ok(Err(MoveFolderError::FolderConflict {
                        error: ConflictError { path },
                    }))
                }
                Err(ApiError::InvalidName { message }) => {
                    return Ok(Err(MoveFolderError::BadFolderPath {
                        error: BadPath { message },
                    }))
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
