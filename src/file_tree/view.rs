use super::extension::strip_file_extension;
use super::folder_share::{ShareOverview, SharedVia};
use super::path::TreePath;
use super::selection::{file_check, folder_check, CheckState};
use super::tree::{FileTree, FolderContents};

/// Expansion key of the built-in `Demo/` row. It ends in `/`, so it can never
/// equal a valid user folder path (a trailing separator is an empty segment).
pub const DEMO_EXPANSION_KEY: &str = "Demo/";
const DEMO_FOLDER_NAME: &str = "Demo";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TreeViewRowKind {
    Folder,
    File,
    /// The read-only `Demo/` folder row.
    DemoFolder,
    /// A read-only demo file; its `key` is the bare demo file name.
    DemoFile,
    /// Placeholder shown under an expanded folder that holds nothing.
    EmptyFolderHint,
}

/// What the view depends on besides the tree itself. All raw UI state: the
/// view decides which rows are visible and which folders count as open.
#[derive(Debug, Clone, Copy)]
pub struct TreeViewInput<'a> {
    /// Path (or bare demo file name) of the file being edited.
    pub active: &'a str,
    /// Expansion keys the user opened (folder paths, plus [`DEMO_EXPANSION_KEY`]).
    pub expanded: &'a [String],
    pub demo_file_names: &'a [String],
    /// The owner's live links; drives the shared marker and the "via" tags.
    pub shares: &'a ShareOverview,
    /// What the search box holds. Blank shows the whole tree; otherwise only
    /// the rows whose name contains it (ignoring case), with their folders.
    pub query: &'a str,
    /// File paths ticked in the browser's multi-select; drives the checkboxes.
    pub selected: &'a [String],
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TreeViewRow {
    /// Folder path, file path, bare demo file name, or [`DEMO_EXPANSION_KEY`].
    /// For a folder this is also its expansion key.
    pub key: String,
    pub name: String,
    /// 0 at the top level.
    pub depth: usize,
    pub kind: TreeViewRowKind,
    /// Open folder (always false for files). The ancestors of the active file
    /// are open regardless of `expanded`.
    pub expanded: bool,
    /// Files at any depth below a folder row; 0 for the other kinds.
    pub file_count: usize,
    pub is_active: bool,
    /// For a folder row: the folder's own live link.
    pub share_id: Option<String>,
    /// The innermost shared folder this row lies inside, if any.
    pub shared_via: Option<SharedVia>,
    /// The part of the displayed name the search query matched, if any.
    pub name_match: Option<NameMatch>,
    /// The row's checkbox; `None` for rows that cannot be selected (demo
    /// files and the empty-folder hint).
    pub check: Option<CheckState>,
    /// Keys of the folder rows this row lies inside, outermost first; empty
    /// at the top level.
    pub ancestor_keys: Vec<String>,
}

/// A half-open range of UTF-16 code units in a row's displayed name (without
/// the `.poentaro` extension), the unit JavaScript string indices use.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NameMatch {
    pub start: usize,
    pub end: usize,
}

/// The active file's location split for the toolbar: its folder names
/// (outermost first) and its own name.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActiveLabel {
    pub folder_names: Vec<String>,
    pub file_name: String,
    /// Full path of the folder holding the file, where new files and folders
    /// go "next to" it; `None` at the top level and for read-only demo files.
    pub folder_path: Option<String>,
}

impl FileTree {
    /// The visible rows of the file list: the user's tree (collapsed folders
    /// hide their contents), then the `Demo/` folder last. Folders and
    /// files are in [`FileTree::rows`] order.
    pub fn view_rows(&self, input: &TreeViewInput) -> Vec<TreeViewRow> {
        let active_path = TreePath::parse(input.active).ok();
        let query = input.query.trim().to_lowercase();
        let filter = (!query.is_empty()).then_some(query.as_str());
        folder_rows(&self.root, 0, &[], input, active_path.as_ref(), filter)
            .into_iter()
            .chain(demo_rows(input, filter))
            .map(|row| TreeViewRow {
                name_match: filter
                    .filter(|_| row.kind != TreeViewRowKind::EmptyFolderHint)
                    .and_then(|query| name_match(&row.name, query)),
                ..row
            })
            .collect()
    }
}

fn folder_rows(
    contents: &FolderContents,
    depth: usize,
    ancestor_keys: &[String],
    input: &TreeViewInput,
    active_path: Option<&TreePath>,
    filter: Option<&str>,
) -> Vec<TreeViewRow> {
    contents
        .folders
        .iter()
        .filter(|folder| {
            filter.is_none_or(|query| {
                name_matches(&folder.name, query) || has_match(&folder.contents, query)
            })
        })
        .flat_map(|folder| {
            // While searching, every folder that is shown is open. A folder
            // matching by its own name shows everything inside it.
            let expanded = filter.is_some()
                || input.expanded.iter().any(|key| key == folder.path.as_str())
                || active_path.is_some_and(|active| active.is_within(&folder.path));
            let child_filter = filter.filter(|query| !name_matches(&folder.name, query));
            let header = TreeViewRow {
                key: folder.path.as_str().to_string(),
                name: folder.name.clone(),
                depth,
                kind: TreeViewRowKind::Folder,
                expanded,
                file_count: count_files(&folder.contents),
                is_active: false,
                share_id: input
                    .shares
                    .folder_share(&folder.path)
                    .map(|share| share.share_id.clone()),
                shared_via: input.shares.shared_via(&folder.path),
                name_match: None,
                check: Some(folder_check(&folder.contents, input.selected)),
                ancestor_keys: ancestor_keys.to_vec(),
            };
            let child_ancestor_keys: Vec<String> = ancestor_keys
                .iter()
                .cloned()
                .chain(std::iter::once(folder.path.as_str().to_string()))
                .collect();
            let children = if !expanded {
                vec![]
            } else if folder.contents.folders.is_empty() && folder.contents.files.is_empty() {
                vec![hint_row(depth + 1, child_ancestor_keys)]
            } else {
                folder_rows(
                    &folder.contents,
                    depth + 1,
                    &child_ancestor_keys,
                    input,
                    active_path,
                    child_filter,
                )
            };
            std::iter::once(header).chain(children)
        })
        .chain(
            contents
                .files
                .iter()
                .filter(|file| filter.is_none_or(|query| name_matches(&file.name, query)))
                .map(|file| TreeViewRow {
                    key: file.path.as_str().to_string(),
                    name: file.name.clone(),
                    depth,
                    kind: TreeViewRowKind::File,
                    expanded: false,
                    file_count: 0,
                    is_active: active_path.is_some_and(|active| *active == file.path),
                    share_id: None,
                    shared_via: input.shares.shared_via(&file.path),
                    name_match: None,
                    check: Some(file_check(&file.path, input.selected)),
                    ancestor_keys: ancestor_keys.to_vec(),
                }),
        )
        .collect()
}

fn demo_rows(input: &TreeViewInput, filter: Option<&str>) -> Vec<TreeViewRow> {
    if input.demo_file_names.is_empty() {
        return vec![];
    }
    let demo_name_matches = filter.is_some_and(|query| name_matches(DEMO_FOLDER_NAME, query));
    let file_filter = filter.filter(|_| !demo_name_matches);
    let shown_files: Vec<&String> = input
        .demo_file_names
        .iter()
        .filter(|name| file_filter.is_none_or(|query| name_matches(name, query)))
        .collect();
    if shown_files.is_empty() {
        return vec![];
    }
    let active_is_demo = input
        .demo_file_names
        .iter()
        .any(|name| name == input.active);
    let expanded = filter.is_some()
        || active_is_demo
        || input.expanded.iter().any(|key| key == DEMO_EXPANSION_KEY);
    let header = TreeViewRow {
        key: DEMO_EXPANSION_KEY.to_string(),
        name: DEMO_FOLDER_NAME.to_string(),
        depth: 0,
        kind: TreeViewRowKind::DemoFolder,
        expanded,
        file_count: input.demo_file_names.len(),
        is_active: false,
        share_id: None,
        shared_via: None,
        name_match: None,
        check: None,
        ancestor_keys: vec![],
    };
    let files = shown_files
        .into_iter()
        .filter(|_| expanded)
        .map(|name| TreeViewRow {
            key: name.clone(),
            name: name.clone(),
            depth: 1,
            kind: TreeViewRowKind::DemoFile,
            expanded: false,
            file_count: 0,
            is_active: name == input.active,
            share_id: None,
            shared_via: None,
            name_match: None,
            check: None,
            ancestor_keys: vec![DEMO_EXPANSION_KEY.to_string()],
        });
    std::iter::once(header).chain(files).collect()
}

fn hint_row(depth: usize, ancestor_keys: Vec<String>) -> TreeViewRow {
    TreeViewRow {
        key: String::new(),
        name: "Empty folder".to_string(),
        depth,
        kind: TreeViewRowKind::EmptyFolderHint,
        expanded: false,
        file_count: 0,
        is_active: false,
        share_id: None,
        shared_via: None,
        name_match: None,
        check: None,
        ancestor_keys,
    }
}

/// Whether a file or folder name contains the (already lowercased) query.
/// The extension is not part of the name the user sees.
fn name_matches(name: &str, query: &str) -> bool {
    strip_file_extension(name).to_lowercase().contains(query)
}

/// Where the (already lowercased) query occurs in the name as the user sees
/// it, so the list can highlight it. `None` when it does not occur, or when
/// lowercasing changes the name's length (offsets could not be mapped back).
fn name_match(name: &str, query: &str) -> Option<NameMatch> {
    let shown = strip_file_extension(name);
    let lowered = shown.to_lowercase();
    if lowered.chars().count() != shown.chars().count() {
        return None;
    }
    let start_byte = lowered.find(query)?;
    let utf16_len = |text: &str| text.encode_utf16().count();
    let start = utf16_len(&lowered[..start_byte]);
    Some(NameMatch {
        start,
        end: start + utf16_len(query),
    })
}

/// Whether anything inside `contents`, at any depth, matches the query.
fn has_match(contents: &FolderContents, query: &str) -> bool {
    contents
        .files
        .iter()
        .any(|file| name_matches(&file.name, query))
        || contents
            .folders
            .iter()
            .any(|folder| name_matches(&folder.name, query) || has_match(&folder.contents, query))
}

fn count_files(contents: &FolderContents) -> usize {
    contents.files.len()
        + contents
            .folders
            .iter()
            .map(|folder| count_files(&folder.contents))
            .sum::<usize>()
}

/// Splits the active file for the toolbar button. A demo file shows under
/// `Demo`; a path that fails to parse is shown whole.
pub fn active_label(active: &str, demo_file_names: &[String]) -> ActiveLabel {
    if demo_file_names.iter().any(|name| name == active) {
        return ActiveLabel {
            folder_names: vec![DEMO_FOLDER_NAME.to_string()],
            file_name: active.to_string(),
            folder_path: None,
        };
    }
    match TreePath::parse(active) {
        Ok(path) => ActiveLabel {
            folder_names: path
                .ancestors()
                .iter()
                .map(|ancestor| ancestor.name().to_string())
                .collect(),
            file_name: path.name().to_string(),
            folder_path: path.parent().map(|parent| parent.as_str().to_string()),
        },
        Err(_) => ActiveLabel {
            folder_names: vec![],
            file_name: active.to_string(),
            folder_path: None,
        },
    }
}
