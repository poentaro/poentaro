use super::extension::FILE_EXTENSION;
use super::path::{PathError, TreePath};
use std::collections::HashSet;

/// `desired` if free, else the first of `stem 2.ext`, `stem 3.ext`, ... that
/// is free in the same folder. `taken` may hold paths from anywhere (files,
/// folders, binned files); only siblings of `desired` matter. A name with no
/// extension (or only a leading dot) gets `.poentaro`, like the TS it replaces.
/// Fails only when the suffixed path would exceed the length limit.
pub fn unique_path(desired: &TreePath, taken: &HashSet<TreePath>) -> Result<TreePath, PathError> {
    let parent = desired.parent();
    let sibling_names: HashSet<&str> = taken
        .iter()
        .filter(|path| path.parent() == parent)
        .map(TreePath::name)
        .collect();
    if !sibling_names.contains(desired.name()) {
        return Ok(desired.clone());
    }
    let (stem, extension) = split_extension(desired.name());
    let candidate = (2..)
        .map(|n| format!("{stem} {n}{extension}"))
        .find(|candidate| !sibling_names.contains(candidate.as_str()))
        .unwrap_or_default();
    TreePath::child(parent.as_ref(), &candidate)
}

fn split_extension(name: &str) -> (&str, &str) {
    match name.rfind('.') {
        Some(dot) if dot > 0 => name.split_at(dot),
        _ => (name, FILE_EXTENSION),
    }
}

/// Highest numeric suffix [`path_after_collision`] steps over; a longer
/// trailing number is treated as part of the name.
const MAX_COLLISION_SUFFIX: u32 = 10_000;

/// The path to try after the server reported `rejected` as taken, when only
/// the rejected path is known (the cloud outbox delivers long after the
/// edit): the next free numbered sibling in the same folder, keeping the
/// folder prefix. A trailing ` N` on the stem is incremented, so `a.poentaro`
/// becomes `a 2.poentaro` and `a 2.poentaro` becomes `a 3.poentaro`, not `a 2 2`.
/// Computed through [`unique_path`] with `a` and `a 2` ... `a N` marked taken.
pub fn path_after_collision(rejected: &TreePath) -> Result<TreePath, PathError> {
    let (stem, extension) = split_extension(rejected.name());
    let (desired, taken) = numbered_after(rejected, stem, extension)?;
    unique_path(&desired, &taken)
}

/// The folder counterpart of [`path_after_collision`]: `New` becomes
/// `New 2` and `New 2` becomes `New 3`, with no extension added.
pub fn folder_after_collision(rejected: &TreePath) -> Result<TreePath, PathError> {
    let (desired, taken) = numbered_after(rejected, rejected.name(), "")?;
    unique_folder_path(&desired, &taken)
}

/// The base of `stem` (its trailing ` N` stripped) as a sibling of
/// `rejected`, and every numbered sibling up to the rejected one, marked
/// taken.
fn numbered_after(
    rejected: &TreePath,
    stem: &str,
    extension: &str,
) -> Result<(TreePath, HashSet<TreePath>), PathError> {
    let parent = rejected.parent();
    let (base_stem, rejected_number) = stem
        .rsplit_once(' ')
        .and_then(|(base, digits)| Some((base, digits.parse::<u32>().ok()?)))
        .filter(|(base, number)| !base.is_empty() && (2..=MAX_COLLISION_SUFFIX).contains(number))
        .unwrap_or((stem, 1));
    let taken: HashSet<TreePath> = std::iter::once(base_stem.to_string())
        .chain((2..=rejected_number).map(|n| format!("{base_stem} {n}")))
        .map(|taken_stem| TreePath::child(parent.as_ref(), &format!("{taken_stem}{extension}")))
        .collect::<Result<_, _>>()?;
    let desired = TreePath::child(parent.as_ref(), &format!("{base_stem}{extension}"))?;
    Ok((desired, taken))
}

/// `desired` if free, else `name 2`, `name 3`, ... among the siblings of
/// `desired`: the folder counterpart of [`unique_path`] (a folder name has no
/// extension, so none is appended).
pub fn unique_folder_path(
    desired: &TreePath,
    taken: &HashSet<TreePath>,
) -> Result<TreePath, PathError> {
    let parent = desired.parent();
    let sibling_names: HashSet<&str> = taken
        .iter()
        .filter(|path| path.parent() == parent)
        .map(TreePath::name)
        .collect();
    let name = desired.name();
    let candidate = std::iter::once(name.to_string())
        .chain((2..).map(|n| format!("{name} {n}")))
        .find(|candidate| !sibling_names.contains(candidate.as_str()))
        .unwrap_or_default();
    TreePath::child(parent.as_ref(), &candidate)
}
