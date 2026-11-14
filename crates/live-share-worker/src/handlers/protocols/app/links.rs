//! `links/*` of `poentaro app`: the owner's public links on files and
//! folders.

use poentaro_protocol::app::links::{
    FileLink, FolderLink, Links, StartFileError, StartFolderError, Status, StatusOfFolderError,
    StopFileError, StopFolderError,
};

use super::{AppServer, Outcome};
use crate::api_error::ApiError;
use crate::db;
use crate::folder_share;
use crate::handlers::protocols::internal;
use crate::handlers::shares;
use crate::protocol::ShareStatus;

fn status(share: Option<ShareStatus>) -> Status {
    match share {
        None => Status::NeverShared,
        Some(ShareStatus {
            share_id,
            ended: true,
        }) => Status::Stopped { share_id },
        Some(ShareStatus {
            share_id,
            ended: false,
        }) => Status::Live { share_id },
    }
}

impl poentaro_protocol::app::links::Server for AppServer<'_> {
    async fn start_file(&self, id: String) -> Outcome<String, StartFileError> {
        match shares::start_file_link(self.d1, &self.user_id, &id).await {
            Ok(share_id) => Ok(Ok(share_id)),
            Err(ApiError::NotFound) => {
                let file = db::get_file_by_id(self.d1, &id, &self.user_id).await?;
                Ok(Err(if file.is_some() {
                    StartFileError::Trashed
                } else {
                    StartFileError::Missing
                }))
            }
            Err(error) => Err(internal(error)),
        }
    }

    async fn stop_file(&self, id: String) -> Outcome<(), StopFileError> {
        match shares::stop_file_link(self.d1, &self.user_id, &id).await {
            Ok(()) => Ok(Ok(())),
            Err(ApiError::NotFound) => Ok(Err(StopFileError::NeverShared)),
            Err(error) => Err(internal(error)),
        }
    }

    async fn status_of_file(&self, id: String) -> worker::Result<Status> {
        Ok(status(
            db::get_share_status_by_file(self.d1, &id, &self.user_id).await?,
        ))
    }

    async fn start_folder(&self, path: String) -> Outcome<String, StartFolderError> {
        let started = match folder_share::validate_folder_path(&path) {
            Ok(folder) => shares::start_folder_link(self.d1, &self.user_id, folder.as_str()).await,
            Err(error) => Err(error),
        };
        match started {
            Ok(share_id) => Ok(Ok(share_id)),
            Err(ApiError::NotFound) => Ok(Err(StartFolderError::NoSuchFolder)),
            Err(ApiError::SharedFolderTooLarge { message }) => {
                Ok(Err(StartFolderError::TooLarge { message }))
            }
            Err(ApiError::InvalidName { message }) => {
                Ok(Err(StartFolderError::InvalidPath { message }))
            }
            Err(error) => Err(internal(error)),
        }
    }

    async fn stop_folder(&self, path: String) -> Outcome<(), StopFolderError> {
        let stopped = match folder_share::validate_folder_path(&path) {
            Ok(folder) => shares::stop_folder_link(self.d1, &self.user_id, folder.as_str()).await,
            Err(error) => Err(error),
        };
        match stopped {
            Ok(()) => Ok(Ok(())),
            Err(ApiError::NotFound) => Ok(Err(StopFolderError::NeverShared)),
            Err(ApiError::InvalidName { message }) => {
                Ok(Err(StopFolderError::InvalidPath { message }))
            }
            Err(error) => Err(internal(error)),
        }
    }

    async fn status_of_folder(&self, path: String) -> Outcome<Status, StatusOfFolderError> {
        match folder_share::validate_folder_path(&path) {
            Ok(folder) => Ok(Ok(status(
                db::get_folder_share_status(self.d1, &self.user_id, folder.as_str()).await?,
            ))),
            Err(ApiError::InvalidName { message }) => {
                Ok(Err(StatusOfFolderError::InvalidPath { message }))
            }
            Err(error) => Err(internal(error)),
        }
    }

    async fn list(&self) -> worker::Result<Links> {
        let files = db::list_live_file_shares(self.d1, &self.user_id).await?;
        let folders = db::list_live_folder_shares(self.d1, &self.user_id).await?;
        Ok(Links {
            files: files
                .into_iter()
                .map(|entry| FileLink {
                    share_id: entry.share_id,
                    file_id: entry.file_id,
                    path: entry.name,
                })
                .collect(),
            folders: folders
                .into_iter()
                .map(|entry| FolderLink {
                    share_id: entry.share_id,
                    path: entry.folder,
                })
                .collect(),
        })
    }
}
