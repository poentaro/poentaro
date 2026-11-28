//! The link endpoints of `api`: the owner's public links on files and
//! folders.

use poentaro_protocol::api::{BadPath, FileError, FolderLookupError, LookupError};
use poentaro_protocol::api::{FileLink, FolderLink, Links, Status};
use poentaro_protocol::api::{ShareFolderError, StopSharingError, StopSharingFolderError};

use super::{AppServer, Outcome};
use crate::api_error::ApiError;
use crate::db;
use crate::folder_share;
use crate::handlers::protocols::internal;
use crate::handlers::shares;
use crate::model::ShareStatus;

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

impl AppServer<'_> {
    pub(in crate::handlers::protocols) async fn start_sharing_file(
        &self,
        id: String,
    ) -> Outcome<String, FileError> {
        match shares::start_file_link(self.d1, &self.user_id, &id).await {
            Ok(share_id) => Ok(Ok(share_id)),
            Err(ApiError::NotFound) => {
                let file = db::get_file_by_id(self.d1, &id, &self.user_id).await?;
                Ok(Err(if file.is_some() {
                    FileError::Trashed
                } else {
                    FileError::NotFound {
                        error: LookupError {},
                    }
                }))
            }
            Err(error) => Err(internal(error)),
        }
    }

    pub(in crate::handlers::protocols) async fn stop_sharing_file(
        &self,
        id: String,
    ) -> Outcome<(), StopSharingError> {
        match shares::stop_file_link(self.d1, &self.user_id, &id).await {
            Ok(()) => Ok(Ok(())),
            Err(ApiError::NotFound) => Ok(Err(StopSharingError {})),
            Err(error) => Err(internal(error)),
        }
    }

    pub(in crate::handlers::protocols) async fn sharing_status_of_file(
        &self,
        id: String,
    ) -> worker::Result<Status> {
        Ok(status(
            db::get_share_status_by_file(self.d1, &id, &self.user_id).await?,
        ))
    }

    pub(in crate::handlers::protocols) async fn start_sharing_folder(
        &self,
        path: String,
    ) -> Outcome<String, ShareFolderError> {
        let started = match folder_share::validate_folder_path(&path) {
            Ok(folder) => shares::start_folder_link(self.d1, &self.user_id, folder.as_str()).await,
            Err(error) => Err(error),
        };
        match started {
            Ok(share_id) => Ok(Ok(share_id)),
            Err(ApiError::NotFound) => Ok(Err(ShareFolderError::SharedFolderNotFound {
                error: FolderLookupError {},
            })),
            Err(ApiError::SharedFolderTooLarge { message }) => {
                Ok(Err(ShareFolderError::FolderTooLarge { message }))
            }
            Err(ApiError::InvalidName { message }) => {
                Ok(Err(ShareFolderError::BadSharedFolderPath {
                    error: BadPath { message },
                }))
            }
            Err(error) => Err(internal(error)),
        }
    }

    pub(in crate::handlers::protocols) async fn stop_sharing_folder(
        &self,
        path: String,
    ) -> Outcome<(), StopSharingFolderError> {
        let stopped = match folder_share::validate_folder_path(&path) {
            Ok(folder) => shares::stop_folder_link(self.d1, &self.user_id, folder.as_str()).await,
            Err(error) => Err(error),
        };
        match stopped {
            Ok(()) => Ok(Ok(())),
            Err(ApiError::NotFound) => Ok(Err(StopSharingFolderError::FolderNeverShared {
                error: StopSharingError {},
            })),
            Err(ApiError::InvalidName { message }) => {
                Ok(Err(StopSharingFolderError::BadStoppedFolderPath {
                    error: BadPath { message },
                }))
            }
            Err(error) => Err(internal(error)),
        }
    }

    pub(in crate::handlers::protocols) async fn sharing_status_of_folder(
        &self,
        path: String,
    ) -> Outcome<Status, BadPath> {
        match folder_share::validate_folder_path(&path) {
            Ok(folder) => Ok(Ok(status(
                db::get_folder_share_status(self.d1, &self.user_id, folder.as_str()).await?,
            ))),
            Err(ApiError::InvalidName { message }) => Ok(Err(BadPath { message })),
            Err(error) => Err(internal(error)),
        }
    }

    pub(in crate::handlers::protocols) async fn list_shared_links(&self) -> worker::Result<Links> {
        let files = db::list_live_file_shares(self.d1, &self.user_id).await?;
        let folders = db::list_live_folder_shares(self.d1, &self.user_id).await?;
        Ok(Links {
            files: files
                .into_iter()
                .map(|entry| FileLink {
                    share_id: entry.share_id,
                    file_id: entry.file_id,
                    name: entry.name,
                })
                .collect(),
            folders: folders
                .into_iter()
                .map(|entry| FolderLink {
                    share_id: entry.share_id,
                    folder: entry.folder,
                })
                .collect(),
        })
    }
}
