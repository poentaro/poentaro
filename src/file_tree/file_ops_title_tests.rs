use super::*;

fn context() -> FileOperationContext {
    FileOperationContext {
        demo_files: Vec::new(),
        new_file_id: "new-id".to_string(),
    }
}

fn created(folder: Option<&str>, name: &str, content: &str) -> String {
    let folder = folder.map(|raw| TreePath::parse(raw).unwrap());
    let next = create_file(
        &FileStoreState::default(),
        folder.as_ref(),
        name,
        content,
        &context(),
    )
    .unwrap();
    next.user_files[&next.active].clone()
}

#[test]
fn a_new_file_without_a_title_is_titled_by_its_name() {
    let content = created(None, " Amazing Grace ", "metadata { author \"\" },\n");
    assert!(content.contains(r#"title "Amazing Grace""#), "{content}");
}

#[test]
fn the_title_drops_the_extension_and_folder() {
    let content = created(Some("Hymns"), "Abide.poentaro", "metadata { },\n");
    assert!(content.contains(r#"title "Abide""#), "{content}");
}

#[test]
fn a_blank_title_is_replaced_by_the_name() {
    let content = created(None, "Song", "metadata { title \"  \" },\n");
    assert!(content.contains(r#"title "Song""#), "{content}");
}

#[test]
fn a_title_already_written_is_kept() {
    let content = created(None, "Song", "metadata { title \"Kept\" },\n");
    assert!(content.contains(r#"title "Kept""#), "{content}");
    assert!(!content.contains(r#"title "Song""#), "{content}");
}

#[test]
fn the_new_file_template_is_titled_by_its_name() {
    let template = include_str!("../../new_file_template.poentaro");
    let content = created(None, "Song", template);
    assert!(content.contains(r#"title "Song""#), "{content}");
}
