//! The methods of `poentaro.protocol.anyx`, over D1, acting as the key's
//! owner on the owner's own files. Files are addressed by path; a write is
//! the same compare-and-swap as the web app's save, and goes into the file's
//! version history the same way.

use std::cell::RefCell;

use poentaro_protocol::{
    CreateContentError, File, FileSummary, IntoValue, ReadError, Saved, Server, Value,
    WriteAtRevisionContentError,
};
use worker::D1Database;

use crate::agent::{file_id, new_file_path_problem, score_diagnostics};
use crate::db;
use crate::files::{self, ContentWriteAttempt, StoredFile};
use crate::handlers::files::{proposed_name, record_version};
use crate::handlers::{is_unique_constraint_violation, now_millis};

type Outcome<T, E> = worker::Result<Result<T, E>>;

pub(super) struct AgentFiles<'a> {
    pub(super) d1: &'a D1Database,
    pub(super) user_id: String,
    /// Each call as its method and how it ended, for `app_log`.
    pub(super) calls: RefCell<Vec<String>>,
}

impl AgentFiles<'_> {
    fn note<T, E: IntoValue + Clone>(&self, method: &str, outcome: &Outcome<T, E>) {
        let ending = match outcome {
            Ok(Ok(_)) => "ok".to_string(),
            Ok(Err(error)) => match error.clone().into_value() {
                Value::Variant { shape, .. } => shape,
                _ => "error".to_string(),
            },
            Err(error) => internal_error(error),
        };
        self.calls.borrow_mut().push(format!("{method}: {ending}"));
    }

    async fn file_at(&self, path: &str) -> worker::Result<Option<StoredFile>> {
        db::get_file_by_name(self.d1, &self.user_id, path).await
    }

    async fn read_file(&self, path: &str) -> Outcome<File, ReadError> {
        Ok(match self.file_at(path).await? {
            None => Err(ReadError::Missing),
            Some(file) if file.trashed_at.is_some() => Err(ReadError::Trashed),
            Some(file) => Ok(File {
                path: file.name,
                revision: file.revision,
                content: file.content,
            }),
        })
    }

    async fn write_file(
        &self,
        path: &str,
        revision: i64,
        content: String,
    ) -> Outcome<Saved, WriteAtRevisionContentError> {
        let previous = match self.file_at(path).await? {
            None => return Ok(Err(WriteAtRevisionContentError::Missing)),
            Some(file) if file.trashed_at.is_some() => {
                return Ok(Err(WriteAtRevisionContentError::Trashed))
            }
            Some(file) => file,
        };
        if previous.revision != revision {
            return Ok(Err(conflict(previous)));
        }
        let diagnostics = score_diagnostics(&content);
        if !diagnostics.errors.is_empty() {
            return Ok(Err(WriteAtRevisionContentError::Rejected {
                diagnostics: diagnostics.errors,
            }));
        }
        let attempt = db::update_file_content(
            self.d1,
            &previous.id,
            &self.user_id,
            &content,
            revision,
            now_millis(),
        )
        .await?;
        match attempt {
            ContentWriteAttempt::Applied => {
                let new_revision = revision + 1;
                if let Err(error) = record_version(self.d1, &previous, &content, new_revision).await
                {
                    worker::console_error!("file_versions: failed to record: {error}");
                }
                Ok(Ok(Saved {
                    revision: new_revision,
                    warnings: diagnostics.warnings,
                }))
            }
            // Someone saved, trashed or deleted it since it was read above.
            ContentWriteAttempt::RowsAffectedZero => Ok(Err(
                match db::get_file_by_id(self.d1, &previous.id, &self.user_id).await? {
                    None => WriteAtRevisionContentError::Missing,
                    Some(file) if file.trashed_at.is_some() => WriteAtRevisionContentError::Trashed,
                    Some(file) => conflict(file),
                },
            )),
        }
    }

    async fn create_file(
        &self,
        path: String,
        content: String,
    ) -> Outcome<Saved, CreateContentError> {
        if let Some(message) = new_file_path_problem(&path) {
            return Ok(Err(CreateContentError::InvalidPath { message }));
        }
        if let Some(existing) = self.file_at(&path).await? {
            return Ok(Err(already_exists(&existing)));
        }
        let id = file_id(random_id_bytes()?);
        let live = db::list_live_file_names(self.d1, &self.user_id).await?;
        if let Some(clash) = files::find_file_folder_conflict(&live, &proposed_name(&id, &path)) {
            let message = format!("found `{clash}` as both a file and a folder; expected a path that is only one of them");
            return Ok(Err(CreateContentError::InvalidPath { message }));
        }
        let diagnostics = score_diagnostics(&content);
        if !diagnostics.errors.is_empty() {
            return Ok(Err(CreateContentError::Rejected {
                diagnostics: diagnostics.errors,
            }));
        }
        let now = now_millis();
        let created = StoredFile {
            id,
            owner_user_id: self.user_id.clone(),
            name: path,
            content,
            revision: 0,
            trashed_at: None,
            created_at: now,
            updated_at: now,
        };
        match db::insert_file(self.d1, &created).await {
            Ok(()) => Ok(Ok(Saved {
                revision: created.revision,
                warnings: diagnostics.warnings,
            })),
            // Lost a race with another create of the same path.
            Err(error) if is_unique_constraint_violation(&error) => {
                match self.file_at(&created.name).await? {
                    Some(existing) => Ok(Err(already_exists(&existing))),
                    None => Err(error),
                }
            }
            Err(error) => Err(error),
        }
    }
}

fn internal_error(error: &worker::Error) -> String {
    format!("internal error: {error}")
}

fn conflict(current: StoredFile) -> WriteAtRevisionContentError {
    WriteAtRevisionContentError::ConflictAtRevisionContent {
        revision: current.revision,
        content: current.content,
    }
}

fn already_exists(existing: &StoredFile) -> CreateContentError {
    CreateContentError::AlreadyExistsAtRevision {
        revision: existing.revision,
    }
}

fn random_id_bytes() -> worker::Result<[u8; 16]> {
    crate::crypto::random_bytes(16)?
        .try_into()
        .map_err(|_| worker::Error::RustError("expected 16 random bytes".to_string()))
}

impl Server for AgentFiles<'_> {
    type Error = worker::Error;

    async fn list_files(&self) -> worker::Result<Vec<FileSummary>> {
        let listed = db::list_files_by_owner(self.d1, &self.user_id).await;
        let ending = match &listed {
            Ok(_) => "ok".to_string(),
            Err(error) => internal_error(error),
        };
        self.calls
            .borrow_mut()
            .push(format!("list files: {ending}"));
        let mut summaries: Vec<FileSummary> = listed?
            .into_iter()
            .filter(|file| file.trashed_at.is_none())
            .map(|file| FileSummary {
                path: file.name,
                revision: file.revision,
            })
            .collect();
        summaries.sort_by(|left, right| left.path.cmp(&right.path));
        Ok(summaries)
    }

    async fn read(&self, path: String) -> Outcome<File, ReadError> {
        let outcome = self.read_file(&path).await;
        self.note("read _", &outcome);
        outcome
    }

    async fn write_at_revision_content(
        &self,
        path: String,
        revision: i64,
        content: String,
    ) -> Outcome<Saved, WriteAtRevisionContentError> {
        let outcome = self.write_file(&path, revision, content).await;
        self.note("write _ at revision _ content _", &outcome);
        outcome
    }

    async fn create_content(
        &self,
        path: String,
        content: String,
    ) -> Outcome<Saved, CreateContentError> {
        let outcome = self.create_file(path, content).await;
        self.note("create _ content _", &outcome);
        outcome
    }
}
