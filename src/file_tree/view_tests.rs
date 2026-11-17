use super::*;

fn path(raw: &str) -> TreePath {
    TreePath::parse(raw).unwrap()
}

fn strings(raw: &[&str]) -> Vec<String> {
    raw.iter().map(|raw| (*raw).to_string()).collect()
}

/// One line per row: indent, name, `/` + count for folders, markers.
fn render(rows: &[TreeViewRow]) -> Vec<String> {
    rows.iter()
        .map(|row| {
            let marker = match row.kind {
                TreeViewRowKind::Folder | TreeViewRowKind::DemoFolder => format!(
                    "/ {} {}",
                    row.file_count,
                    if row.expanded { "open" } else { "closed" }
                ),
                _ => String::new(),
            };
            format!(
                "{}{}{}{}",
                "  ".repeat(row.depth),
                row.name,
                marker,
                if row.is_active { " *" } else { "" }
            )
        })
        .collect()
}

fn tree() -> FileTree {
    FileTree::build(
        &[
            path("Hymns/Advent/O Come.poentaro"),
            path("Hymns/Advent/Rejoice.poentaro"),
            path("Hymns/Amazing.poentaro"),
            path("Scratch.poentaro"),
        ],
        &[path("Empty")],
    )
    .unwrap()
}

#[test]
fn folders_are_collapsed_by_default_and_show_recursive_file_counts() {
    let rows = tree().view_rows(&TreeViewInput {
        active: "Scratch.poentaro",
        expanded: &[],
        demo_file_names: &strings(&["01.poentaro", "02.poentaro"]),
        shares: &ShareOverview::default(),
        query: "",
        selected: &[],
    });
    assert_eq!(
        render(&rows),
        vec![
            "Empty/ 0 closed",
            "Hymns/ 3 closed",
            "Scratch.poentaro *",
            "Demo/ 2 closed",
        ]
    );
}

#[test]
fn an_expanded_folder_shows_its_children_and_remembers_nothing_else() {
    let rows = tree().view_rows(&TreeViewInput {
        active: "Scratch.poentaro",
        expanded: &strings(&["Hymns"]),
        demo_file_names: &[],
        shares: &ShareOverview::default(),
        query: "",
        selected: &[],
    });
    assert_eq!(
        render(&rows),
        vec![
            "Empty/ 0 closed",
            "Hymns/ 3 open",
            "  Advent/ 2 closed",
            "  Amazing.poentaro",
            "Scratch.poentaro *",
        ]
    );
}

#[test]
fn ancestors_of_the_active_file_are_open_even_when_not_expanded() {
    let rows = tree().view_rows(&TreeViewInput {
        active: "Hymns/Advent/Rejoice.poentaro",
        expanded: &[],
        demo_file_names: &[],
        shares: &ShareOverview::default(),
        query: "",
        selected: &[],
    });
    assert_eq!(
        render(&rows),
        vec![
            "Empty/ 0 closed",
            "Hymns/ 3 open",
            "  Advent/ 2 open",
            "    O Come.poentaro",
            "    Rejoice.poentaro *",
            "  Amazing.poentaro",
            "Scratch.poentaro",
        ]
    );
}

#[test]
fn an_expanded_empty_folder_gets_a_hint_row() {
    let rows = tree().view_rows(&TreeViewInput {
        active: "Scratch.poentaro",
        expanded: &strings(&["Empty"]),
        demo_file_names: &[],
        shares: &ShareOverview::default(),
        query: "",
        selected: &[],
    });
    assert_eq!(rows[0].kind, TreeViewRowKind::Folder);
    assert_eq!(rows[1].kind, TreeViewRowKind::EmptyFolderHint);
    assert_eq!(rows[1].depth, 1);
}

#[test]
fn demo_is_last_collapsed_by_default_and_opens_by_key_or_active_file() {
    let demo = strings(&["01.poentaro", "02.poentaro"]);
    let opened = tree().view_rows(&TreeViewInput {
        active: "Scratch.poentaro",
        expanded: &strings(&[DEMO_EXPANSION_KEY]),
        demo_file_names: &demo,
        shares: &ShareOverview::default(),
        query: "",
        selected: &[],
    });
    assert_eq!(
        &render(&opened)[3..],
        ["Demo/ 2 open", "  01.poentaro", "  02.poentaro"]
    );
    assert_eq!(opened[4].kind, TreeViewRowKind::DemoFile);

    let active_demo = tree().view_rows(&TreeViewInput {
        active: "02.poentaro",
        expanded: &[],
        demo_file_names: &demo,
        shares: &ShareOverview::default(),
        query: "",
        selected: &[],
    });
    assert_eq!(
        &render(&active_demo)[3..],
        ["Demo/ 2 open", "  01.poentaro", "  02.poentaro *"]
    );
}

#[test]
fn active_label_splits_folders_from_the_file_name() {
    assert_eq!(
        active_label("Hymns/Advent/O Come.poentaro", &[]),
        ActiveLabel {
            folder_names: strings(&["Hymns", "Advent"]),
            file_name: "O Come.poentaro".to_string(),
            folder_path: Some("Hymns/Advent".to_string()),
        }
    );
    assert_eq!(
        active_label("a.poentaro", &[]).folder_names,
        Vec::<String>::new()
    );
    assert_eq!(
        active_label("01.poentaro", &strings(&["01.poentaro"])).folder_names,
        strings(&["Demo"])
    );
    assert_eq!(active_label("a.poentaro", &[]).folder_path, None);
    assert_eq!(
        active_label("01.poentaro", &strings(&["01.poentaro"])).folder_path,
        None
    );
}

#[test]
fn rows_inside_a_shared_folder_carry_the_marker_and_the_via_tag() {
    let shares = ShareOverview {
        files: vec![],
        folders: vec![FolderShareLink {
            share_id: "S1".to_string(),
            folder: "Hymns".to_string(),
        }],
    };
    let rows = tree().view_rows(&TreeViewInput {
        active: "Scratch.poentaro",
        expanded: &strings(&["Hymns", "Hymns/Advent"]),
        demo_file_names: &[],
        shares: &shares,
        query: "",
        selected: &[],
    });
    let shared_info = |name: &str| {
        let row = rows.iter().find(|row| row.name == name).unwrap();
        (
            row.share_id.clone(),
            row.shared_via.as_ref().map(|via| via.name.clone()),
        )
    };
    assert_eq!(shared_info("Hymns"), (Some("S1".to_string()), None));
    assert_eq!(shared_info("Advent"), (None, Some("Hymns".to_string())));
    assert_eq!(
        shared_info("O Come.poentaro"),
        (None, Some("Hymns".to_string()))
    );
    assert_eq!(shared_info("Scratch.poentaro"), (None, None));
    assert_eq!(shared_info("Empty"), (None, None));
}

fn searched(query: &str) -> Vec<String> {
    render(&tree().view_rows(&TreeViewInput {
        active: "Scratch.poentaro",
        expanded: &[],
        demo_file_names: &strings(&["01-scale.poentaro", "02-rests.poentaro"]),
        shares: &ShareOverview::default(),
        query,
        selected: &[],
    }))
}

#[test]
fn a_blank_query_shows_the_whole_tree() {
    assert_eq!(searched("   "), searched(""));
}

#[test]
fn a_matching_file_keeps_its_folders_open_and_hides_the_rest() {
    assert_eq!(
        searched("rejoice"),
        vec!["Hymns/ 3 open", "  Advent/ 2 open", "    Rejoice.poentaro"]
    );
}

#[test]
fn a_folder_matching_by_name_shows_everything_inside_it() {
    assert_eq!(
        searched("advent"),
        vec![
            "Hymns/ 3 open",
            "  Advent/ 2 open",
            "    O Come.poentaro",
            "    Rejoice.poentaro"
        ]
    );
}

#[test]
fn the_extension_is_not_searchable() {
    assert_eq!(searched("poentaro"), Vec::<String>::new());
}

#[test]
fn demo_files_are_searched_too() {
    assert_eq!(
        searched("rests"),
        vec!["Demo/ 2 open", "  02-rests.poentaro"]
    );
}

#[test]
fn a_query_matching_nothing_leaves_no_rows() {
    assert_eq!(searched("zzz"), Vec::<String>::new());
}

#[test]
fn a_matching_row_reports_where_the_query_occurs_in_its_displayed_name() {
    let rows = tree().view_rows(&TreeViewInput {
        active: "Scratch.poentaro",
        expanded: &[],
        demo_file_names: &[],
        shares: &ShareOverview::default(),
        query: " JOY ",
        selected: &[],
    });
    assert!(rows.iter().all(|row| row.name_match.is_none()));

    let rows = tree().view_rows(&TreeViewInput {
        active: "Scratch.poentaro",
        expanded: &[],
        demo_file_names: &[],
        shares: &ShareOverview::default(),
        query: "COME",
        selected: &[],
    });
    let matches: Vec<_> = rows
        .iter()
        .map(|row| (row.name.as_str(), row.name_match))
        .collect();
    assert_eq!(
        matches,
        vec![
            ("Hymns", None),
            ("Advent", None),
            ("O Come.poentaro", Some(NameMatch { start: 2, end: 6 })),
        ]
    );
}

#[test]
fn every_row_lists_the_keys_of_the_folders_it_lies_inside() {
    let rows = tree().view_rows(&TreeViewInput {
        active: "Hymns/Advent/Rejoice.poentaro",
        expanded: &strings(&["Empty"]),
        demo_file_names: &strings(&["01.poentaro"]),
        shares: &ShareOverview::default(),
        query: "",
        selected: &[],
    });
    let ancestors: Vec<String> = rows
        .iter()
        .map(|row| format!("{} <- [{}]", row.name, row.ancestor_keys.join(", ")))
        .collect();
    assert_eq!(
        ancestors,
        vec![
            "Empty <- []",
            "Empty folder <- [Empty]",
            "Hymns <- []",
            "Advent <- [Hymns]",
            "O Come.poentaro <- [Hymns, Hymns/Advent]",
            "Rejoice.poentaro <- [Hymns, Hymns/Advent]",
            "Amazing.poentaro <- [Hymns]",
            "Scratch.poentaro <- []",
            "Demo <- []",
        ]
    );
}
