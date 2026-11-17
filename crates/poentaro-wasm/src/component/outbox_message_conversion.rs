//! Conversions between the WIT outbox messages and the pure `cloud_outbox`
//! ones. Conversion only: every decision lives in the `cloud-outbox` crate.
use super::*;
use cloud_outbox::types as co;

impl From<co::Message> for Message {
    fn from(message: co::Message) -> Self {
        match message {
            co::Message::CreateFile { name, content } => {
                Message::CreateFile(CreateFileMessage { name, content })
            }
            co::Message::SaveContent { content } => {
                Message::SaveContent(SaveContentMessage { content })
            }
            co::Message::RenameFile { to } => Message::RenameFile(RenameFileMessage { to }),
            co::Message::RenameFiles { moves, folder } => {
                Message::RenameFiles(RenameFilesMessage {
                    moves: moves.into_iter().map(Into::into).collect(),
                    folder: folder.map(|folder| FolderMove {
                        from: folder.from,
                        to: folder.to,
                    }),
                })
            }
            co::Message::TrashFile => Message::TrashFile,
            co::Message::RestoreFile => Message::RestoreFile,
        }
    }
}

impl From<Message> for co::Message {
    fn from(message: Message) -> Self {
        match message {
            Message::CreateFile(CreateFileMessage { name, content }) => {
                co::Message::CreateFile { name, content }
            }
            Message::SaveContent(SaveContentMessage { content }) => {
                co::Message::SaveContent { content }
            }
            Message::RenameFile(RenameFileMessage { to }) => co::Message::RenameFile { to },
            Message::RenameFiles(RenameFilesMessage { moves, folder }) => {
                co::Message::RenameFiles {
                    moves: moves.into_iter().map(Into::into).collect(),
                    folder: folder.map(|folder| co::FolderMove {
                        from: folder.from,
                        to: folder.to,
                    }),
                }
            }
            Message::TrashFile => co::Message::TrashFile,
            Message::RestoreFile => co::Message::RestoreFile,
        }
    }
}

impl From<co::RenameMove> for RenameMove {
    fn from(rename: co::RenameMove) -> Self {
        RenameMove {
            file_id: rename.file_id,
            to: rename.to,
        }
    }
}

impl From<RenameMove> for co::RenameMove {
    fn from(rename: RenameMove) -> Self {
        co::RenameMove {
            file_id: rename.file_id,
            to: rename.to,
        }
    }
}

impl From<co::NameAdjustment> for NameAdjustment {
    fn from(adjustment: co::NameAdjustment) -> Self {
        NameAdjustment {
            rejected: adjustment.rejected,
            replacement: adjustment.replacement,
        }
    }
}

impl From<NameAdjustment> for co::NameAdjustment {
    fn from(adjustment: NameAdjustment) -> Self {
        co::NameAdjustment {
            rejected: adjustment.rejected,
            replacement: adjustment.replacement,
        }
    }
}
