use super::*;

fn path(raw: &str) -> TreePath {
    TreePath::parse(raw).unwrap()
}

fn strings(raw: &[&str]) -> Vec<String> {
    raw.iter().map(|raw| (*raw).to_string()).collect()
}

fn context() -> FileOperationContext {
    FileOperationContext {
        demo_files: vec![],
        new_file_id: "new-id".to_string(),
    }
}

fn store(files: &[&str]) -> FileStoreState {
    FileStoreState {
        active: files.first().copied().unwrap_or_default().to_string(),
        user_files: files
            .iter()
            .map(|name| (name.to_string(), String::new()))
            .collect(),
        bin: Default::default(),
        file_ids: Default::default(),
        folders: vec![path("Empty")],
    }
}

fn sample() -> FileStoreState {
    store(&[
        "Hymns/Advent/O Come.poentaro",
        "Hymns/Advent/Rejoice.poentaro",
        "Hymns/Amazing.poentaro",
        "Scratch.poentaro",
    ])
}

fn tree() -> FileTree {
    sample().tree().unwrap()
}

fn unit_paths(units: &[SelectionUnit]) -> Vec<String> {
    units.iter().map(|unit| unit.path.clone()).collect()
}

fn checks(selected: &[&str]) -> Vec<(String, Option<CheckState>)> {
    tree()
        .view_rows(&TreeViewInput {
            active: "Scratch.poentaro",
            expanded: &strings(&["Hymns", "Hymns/Advent"]),
            demo_file_names: &[],
            shares: &ShareOverview::default(),
            query: "",
            selected: &strings(selected),
        })
        .into_iter()
        .map(|row| (row.name, row.check))
        .collect()
}

#[test]
fn a_folder_is_checked_only_when_all_its_files_are_selected() {
    use CheckState::*;
    assert_eq!(
        checks(&[
            "Hymns/Advent/O Come.poentaro",
            "Hymns/Advent/Rejoice.poentaro"
        ]),
        vec![
            ("Empty".to_string(), Some(Unchecked)),
            ("Hymns".to_string(), Some(Mixed)),
            ("Advent".to_string(), Some(Checked)),
            ("O Come.poentaro".to_string(), Some(Checked)),
            ("Rejoice.poentaro".to_string(), Some(Checked)),
            ("Amazing.poentaro".to_string(), Some(Unchecked)),
            ("Scratch.poentaro".to_string(), Some(Unchecked)),
        ]
    );
}

#[test]
fn an_empty_folder_is_never_checked() {
    let all = tree().select_all();
    assert_eq!(
        checks(&all.iter().map(String::as_str).collect::<Vec<_>>())[0],
        ("Empty".to_string(), Some(CheckState::Unchecked))
    );
}

#[test]
fn toggling_a_file_flips_only_that_file() {
    let once = tree().toggle_selection(&[], "Scratch.poentaro");
    assert_eq!(once, strings(&["Scratch.poentaro"]));
    assert!(tree()
        .toggle_selection(&once, "Scratch.poentaro")
        .is_empty());
}

#[test]
fn toggling_a_folder_selects_everything_inside_then_clears_it() {
    let selected = tree().toggle_selection(&strings(&["Scratch.poentaro"]), "Hymns");
    assert_eq!(
        selected,
        strings(&[
            "Hymns/Advent/O Come.poentaro",
            "Hymns/Advent/Rejoice.poentaro",
            "Hymns/Amazing.poentaro",
            "Scratch.poentaro",
        ])
    );
    assert_eq!(
        tree().toggle_selection(&selected, "Hymns"),
        strings(&["Scratch.poentaro"])
    );
}

#[test]
fn toggling_a_half_selected_folder_selects_the_rest() {
    let selected = tree().toggle_selection(&strings(&["Hymns/Amazing.poentaro"]), "Hymns");
    assert_eq!(selected.len(), 3);
}

#[test]
fn toggling_drops_selected_paths_that_no_longer_exist() {
    assert_eq!(
        tree().toggle_selection(&strings(&["Gone.poentaro"]), "Scratch.poentaro"),
        strings(&["Scratch.poentaro"])
    );
}

#[test]
fn toggling_an_unknown_key_changes_nothing() {
    assert_eq!(
        tree().toggle_selection(&strings(&["Scratch.poentaro"]), "Nowhere"),
        strings(&["Scratch.poentaro"])
    );
}

#[test]
fn a_fully_selected_folder_is_one_unit_without_its_children() {
    let units = tree().selection_units(&strings(&[
        "Hymns/Advent/O Come.poentaro",
        "Hymns/Advent/Rejoice.poentaro",
        "Hymns/Amazing.poentaro",
        "Scratch.poentaro",
    ]));
    assert_eq!(unit_paths(&units), vec!["Hymns", "Scratch.poentaro"]);
    assert_eq!(units[0].file_count, 3);
    assert!(units[0].is_folder);
}

#[test]
fn a_partly_selected_folder_contributes_its_selected_files() {
    let units = tree().selection_units(&strings(&[
        "Hymns/Advent/O Come.poentaro",
        "Hymns/Advent/Rejoice.poentaro",
    ]));
    assert_eq!(unit_paths(&units), vec!["Hymns/Advent"]);
}

#[test]
fn moving_files_into_a_folder_keeps_their_names() {
    let plan = selection_move_plan(
        &sample(),
        &strings(&["Scratch.poentaro", "Hymns/Amazing.poentaro"]),
        Some("Empty"),
        &context(),
    );
    assert_eq!(plan.error, None);
    assert_eq!(
        plan.moves,
        vec![
            SelectionMove {
                from: "Hymns/Amazing.poentaro".to_string(),
                to: "Empty/Amazing.poentaro".to_string(),
                is_folder: false,
            },
            SelectionMove {
                from: "Scratch.poentaro".to_string(),
                to: "Empty/Scratch.poentaro".to_string(),
                is_folder: false,
            },
        ]
    );
}

#[test]
fn moving_a_whole_folder_moves_it_as_a_folder() {
    let plan = selection_move_plan(
        &sample(),
        &strings(&[
            "Hymns/Advent/O Come.poentaro",
            "Hymns/Advent/Rejoice.poentaro",
        ]),
        Some("Empty"),
        &context(),
    );
    assert_eq!(plan.error, None);
    assert_eq!(
        plan.moves,
        vec![SelectionMove {
            from: "Hymns/Advent".to_string(),
            to: "Empty/Advent".to_string(),
            is_folder: true,
        }]
    );
}

#[test]
fn units_already_in_the_target_are_left_out() {
    let plan = selection_move_plan(&sample(), &strings(&["Scratch.poentaro"]), None, &context());
    assert_eq!(plan.moves, vec![]);
    assert_eq!(
        plan.error,
        Some("Everything selected is already in that folder.".to_string())
    );
}

#[test]
fn two_selected_files_with_the_same_name_clash() {
    let state = store(&[
        "A/Song.poentaro",
        "A/Other.poentaro",
        "B/Song.poentaro",
        "B/Other.poentaro",
    ]);
    let plan = selection_move_plan(
        &state,
        &strings(&["A/Song.poentaro", "B/Song.poentaro"]),
        None,
        &context(),
    );
    assert_eq!(
        plan.error,
        Some("`Song.poentaro` already exists".to_string())
    );
}

#[test]
fn a_clash_with_an_existing_file_is_reported() {
    let state = store(&["A/Song.poentaro", "A/Other.poentaro", "Song.poentaro"]);
    let plan = selection_move_plan(&state, &strings(&["A/Song.poentaro"]), None, &context());
    assert_eq!(
        plan.error,
        Some("`Song.poentaro` already exists".to_string())
    );
}

#[test]
fn a_folder_cannot_move_into_itself() {
    let plan = selection_move_plan(
        &sample(),
        &strings(&[
            "Hymns/Advent/O Come.poentaro",
            "Hymns/Advent/Rejoice.poentaro",
        ]),
        Some("Hymns/Advent"),
        &context(),
    );
    assert_eq!(
        plan.error,
        Some("cannot move `Hymns/Advent` into itself (`Hymns/Advent/Advent`)".to_string())
    );
}

#[test]
fn the_delete_summary_lists_files_and_folders() {
    let summary = selection_delete_summary(
        &sample(),
        &strings(&[
            "Hymns/Advent/O Come.poentaro",
            "Hymns/Advent/Rejoice.poentaro",
            "Scratch.poentaro",
        ]),
        &ShareOverview::default(),
    );
    assert_eq!(
        summary.lines,
        vec!["Hymns/Advent/ (2 files)", "Scratch.poentaro"]
    );
    assert_eq!(
        summary.message,
        "3 files will move to the bin. You can restore them from there."
    );
    assert_eq!(summary.share_warning, None);
}

#[test]
fn the_delete_summary_of_one_file_is_singular() {
    let summary = selection_delete_summary(
        &sample(),
        &strings(&["Scratch.poentaro"]),
        &ShareOverview::default(),
    );
    assert_eq!(
        summary.message,
        "1 file will move to the bin. You can restore it from there."
    );
}

#[test]
fn deleting_a_shared_folder_carries_the_share_warning() {
    let shares = ShareOverview {
        files: vec![],
        folders: vec![FolderShareLink {
            share_id: "s1".to_string(),
            folder: "Hymns".to_string(),
        }],
    };
    let summary = selection_delete_summary(
        &sample(),
        &strings(&[
            "Hymns/Advent/O Come.poentaro",
            "Hymns/Advent/Rejoice.poentaro",
            "Hymns/Amazing.poentaro",
        ]),
        &shares,
    );
    assert!(summary
        .share_warning
        .is_some_and(|warning| warning.contains("shared with a link")));
}
