use super::*;

fn path(raw: &str) -> TreePath {
    TreePath::parse(raw).unwrap()
}

fn store(files: &[&str], folders: &[&str]) -> FileStoreState {
    FileStoreState {
        active: String::new(),
        user_files: files
            .iter()
            .map(|name| (name.to_string(), String::new()))
            .collect(),
        bin: Default::default(),
        file_ids: Default::default(),
        folders: folders.iter().map(|raw| path(raw)).collect(),
    }
}

fn overview(files: &[&str], folders: &[&str]) -> ShareOverview {
    ShareOverview {
        files: files
            .iter()
            .map(|name| FileShareLink {
                share_id: format!("file-{name}"),
                file_id: format!("id-{name}"),
                name: name.to_string(),
            })
            .collect(),
        folders: folders
            .iter()
            .map(|folder| FolderShareLink {
                share_id: format!("folder-{folder}"),
                folder: folder.to_string(),
            })
            .collect(),
    }
}

#[test]
fn a_path_inside_a_shared_folder_names_the_innermost_one() {
    let shares = overview(&[], &["Hymns", "Hymns/Advent"]);
    let via = shares
        .shared_via(&path("Hymns/Advent/Rejoice.poentaro"))
        .unwrap();
    assert_eq!(via.folder, "Hymns/Advent");
    assert_eq!(via.name, "Advent");
    assert_eq!(via.share_id, "folder-Hymns/Advent");
    let outer = shares.shared_via(&path("Hymns/Amazing.poentaro")).unwrap();
    assert_eq!(outer.folder, "Hymns");
}

#[test]
fn a_shared_folder_is_not_inside_itself_and_a_name_prefix_is_not_a_parent() {
    let shares = overview(&[], &["Hymns"]);
    assert_eq!(shares.shared_via(&path("Hymns")), None);
    assert_eq!(shares.shared_via(&path("Hymns 2/x.poentaro")), None);
    assert_eq!(shares.shared_via(&path("Other/x.poentaro")), None);
}

#[test]
fn info_counts_files_and_folders_and_reports_the_own_link() {
    let state = store(
        &[
            "Hymns/a.poentaro",
            "Hymns/Advent/b.poentaro",
            "Hymns/Advent/c.poentaro",
        ],
        &["Hymns/Empty"],
    );
    let shares = overview(&[], &["Hymns"]);
    let info = folder_share_info(&state, &shares, &path("Hymns")).unwrap();
    assert_eq!(info.name, "Hymns");
    assert_eq!(info.file_count, 3);
    assert_eq!(info.folder_count, 2);
    assert_eq!(info.share_id.as_deref(), Some("folder-Hymns"));
    assert_eq!(info.shared_via, None);
    assert_eq!(info.own_link_warning, None);
}

#[test]
fn info_for_a_nested_folder_reports_the_enclosing_share() {
    let state = store(&["Hymns/Advent/b.poentaro"], &[]);
    let shares = overview(&[], &["Hymns"]);
    let info = folder_share_info(&state, &shares, &path("Hymns/Advent")).unwrap();
    assert_eq!(info.share_id, None);
    assert_eq!(info.shared_via.unwrap().name, "Hymns");
}

#[test]
fn info_is_none_for_a_missing_folder() {
    assert_eq!(
        folder_share_info(
            &store(&["a.poentaro"], &[]),
            &ShareOverview::default(),
            &path("Nope")
        ),
        None
    );
}

#[test]
fn files_with_their_own_link_are_listed_relative_to_the_folder() {
    let state = store(
        &[
            "Hymns/a.poentaro",
            "Hymns/Advent/b.poentaro",
            "Other/c.poentaro",
        ],
        &[],
    );
    let shares = overview(&["Hymns/Advent/b.poentaro", "Other/c.poentaro"], &[]);
    let info = folder_share_info(&state, &shares, &path("Hymns")).unwrap();
    assert_eq!(
        info.own_link_warning.as_deref(),
        Some("1 file inside already has its own synced link (Advent/b.poentaro). That link keeps working independently of this one.")
    );
}

#[test]
fn a_long_list_of_own_links_is_cut_short() {
    let names: Vec<String> = (1..=7).map(|n| format!("F/{n}.poentaro")).collect();
    let refs: Vec<&str> = names.iter().map(String::as_str).collect();
    let state = store(&refs, &[]);
    let warning = folder_share_info(&state, &overview(&refs, &[]), &path("F"))
        .unwrap()
        .own_link_warning
        .unwrap();
    assert!(warning
        .starts_with("7 files inside already have their own synced links (1.poentaro, 2.poentaro"));
    assert!(warning.contains("and 2 more"));
}

#[test]
fn delete_warning_is_none_when_nothing_is_shared() {
    assert_eq!(
        folder_delete_share_warning(&ShareOverview::default(), &path("A")),
        None
    );
    assert_eq!(
        folder_delete_share_warning(&overview(&[], &["B"]), &path("A")),
        None
    );
}

#[test]
fn delete_warning_says_the_link_stays_alive_and_shows_an_empty_folder() {
    let warning = folder_delete_share_warning(&overview(&[], &["Hymns"]), &path("Hymns")).unwrap();
    assert!(warning.contains("shared with a link"));
    assert!(warning.contains("empty folder"));
    assert!(warning.contains("link itself keeps working"));
}

#[test]
fn delete_warning_covers_nested_shares_enclosing_shares_and_file_links() {
    let nested = folder_delete_share_warning(&overview(&[], &["A/B"]), &path("A")).unwrap();
    assert!(nested.contains("inside (A/B)"));
    let enclosing = folder_delete_share_warning(&overview(&[], &["A"]), &path("A/B")).unwrap();
    assert!(enclosing.contains("inside the shared folder \"A\""));
    let file_links = folder_delete_share_warning(
        &overview(&["A/x.poentaro", "A/y.poentaro"], &[]),
        &path("A"),
    )
    .unwrap();
    assert!(file_links.contains("2 files inside have their own synced links"));
}
