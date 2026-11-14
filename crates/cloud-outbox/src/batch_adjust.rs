//! Resending a batch rename after the server reported destinations taken.

use crate::types::{FolderMove, Message, NameAdjustment, RenameMove};

/// The moves with each adjusted destination replaced (or the move dropped).
/// `None` when no adjustment applies to any move, so a resend could not
/// change the outcome.
fn adjust_moves(moves: &[RenameMove], adjustments: &[NameAdjustment]) -> Option<Vec<RenameMove>> {
    let adjusted: Vec<RenameMove> = moves
        .iter()
        .filter_map(
            |rename| match adjustments.iter().find(|entry| entry.rejected == rename.to) {
                None => Some(rename.clone()),
                Some(NameAdjustment {
                    replacement: Some(to),
                    ..
                }) => Some(RenameMove {
                    file_id: rename.file_id.clone(),
                    to: to.clone(),
                }),
                Some(NameAdjustment {
                    replacement: None, ..
                }) => None,
            },
        )
        .collect();
    (adjusted != moves).then_some(adjusted)
}

/// A folder's batch whose destination folder was taken moves, whole, to the
/// replacement folder instead. `None` when no adjustment names the folder.
fn adjust_folder(
    moves: &[RenameMove],
    folder: &FolderMove,
    adjustments: &[NameAdjustment],
) -> Option<Message> {
    let replacement = adjustments
        .iter()
        .find(|entry| entry.rejected == folder.to)?
        .replacement
        .clone()?;
    let prefix = format!("{}/", folder.to);
    let moves = moves
        .iter()
        .map(|rename| RenameMove {
            file_id: rename.file_id.clone(),
            to: rename
                .to
                .strip_prefix(&prefix)
                .map_or_else(|| rename.to.clone(), |rest| format!("{replacement}/{rest}")),
        })
        .collect();
    Some(Message::RenameFiles {
        moves,
        folder: Some(FolderMove {
            from: folder.from.clone(),
            to: replacement,
        }),
    })
}

/// The batch with its rejected destinations adjusted: a folder's move to
/// another folder, or else each file's destination on its own (no longer
/// one folder's move).
pub(crate) fn adjust_batch(message: &Message, adjustments: &[NameAdjustment]) -> Option<Message> {
    let Message::RenameFiles { moves, folder } = message else {
        return None;
    };
    folder
        .as_ref()
        .and_then(|folder| adjust_folder(moves, folder, adjustments))
        .or_else(|| {
            adjust_moves(moves, adjustments).map(|moves| Message::RenameFiles {
                moves,
                folder: None,
            })
        })
}
