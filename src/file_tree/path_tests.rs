use super::*;

fn parse(raw: &str) -> Result<TreePath, PathError> {
    TreePath::parse(raw)
}

#[test]
fn accepts_simple_and_nested_paths() {
    assert_eq!(parse("a.poentaro").unwrap().as_str(), "a.poentaro");
    assert_eq!(
        parse("Hymns/Advent/O Come.poentaro").unwrap().as_str(),
        "Hymns/Advent/O Come.poentaro"
    );
}

#[test]
fn rejects_empty_path() {
    assert_eq!(parse(""), Err(PathError::Empty));
}

#[test]
fn rejects_leading_trailing_and_double_slash() {
    assert_eq!(parse("/a"), Err(PathError::EmptySegment));
    assert_eq!(parse("a/"), Err(PathError::EmptySegment));
    assert_eq!(parse("a//b"), Err(PathError::EmptySegment));
    assert_eq!(parse("/"), Err(PathError::EmptySegment));
}

#[test]
fn rejects_whitespace_only_segment() {
    assert_eq!(parse("a/ /b"), Err(PathError::EmptySegment));
}

#[test]
fn rejects_dot_and_dot_dot_segments() {
    for raw in [".", "..", "a/./b", "a/../b", "a/.."] {
        assert!(
            matches!(parse(raw), Err(PathError::RelativeSegment { .. })),
            "{raw}"
        );
    }
}

#[test]
fn allows_dots_inside_names() {
    assert!(parse(".hidden").is_ok());
    assert!(parse("a/..b/c.").is_ok());
    assert!(parse("...").is_ok());
}

#[test]
fn ascii_limit_is_exactly_1024_bytes() {
    assert!(parse(&"a".repeat(1024)).is_ok());
    assert_eq!(
        parse(&"a".repeat(1025)),
        Err(PathError::TooLong { bytes: 1025 })
    );
}

#[test]
fn cjk_limit_counts_bytes_not_characters() {
    // 341 * 3 = 1023 bytes, plus one ASCII byte = exactly 1024.
    let at_limit = format!("{}a", "中".repeat(341));
    assert_eq!(at_limit.len(), 1024);
    assert!(parse(&at_limit).is_ok());
    // One more byte tips it over.
    let over_limit = format!("{}ab", "中".repeat(341));
    assert_eq!(over_limit.len(), 1025);
    assert_eq!(parse(&over_limit), Err(PathError::TooLong { bytes: 1025 }));
    // 342 CJK characters is only 342 chars but 1026 bytes.
    assert_eq!(
        parse(&"中".repeat(342)),
        Err(PathError::TooLong { bytes: 1026 })
    );
}

#[test]
fn limit_counts_separators() {
    let path = format!("{}/{}", "a".repeat(512), "b".repeat(511));
    assert_eq!(path.len(), 1024);
    assert!(parse(&path).is_ok());
    assert!(parse(&format!("{path}b")).is_err());
}

#[test]
fn name_parent_and_ancestors() {
    let path = parse("a/b/c.poentaro").unwrap();
    assert_eq!(path.name(), "c.poentaro");
    assert_eq!(path.parent(), Some(parse("a/b").unwrap()));
    assert_eq!(
        path.ancestors(),
        vec![parse("a").unwrap(), parse("a/b").unwrap()]
    );
    let top = parse("c.poentaro").unwrap();
    assert_eq!(top.parent(), None);
    assert!(top.ancestors().is_empty());
    assert_eq!(
        path.segments().collect::<Vec<_>>(),
        vec!["a", "b", "c.poentaro"]
    );
}

#[test]
fn child_joins_and_validates() {
    let folder = parse("a").unwrap();
    assert_eq!(
        TreePath::child(Some(&folder), "b.poentaro")
            .unwrap()
            .as_str(),
        "a/b.poentaro"
    );
    assert_eq!(TreePath::child(None, "b").unwrap().as_str(), "b");
    assert!(TreePath::child(Some(&folder), "x/y").is_err());
    assert!(TreePath::child(Some(&folder), "..").is_err());
    assert!(TreePath::child(Some(&folder), "").is_err());
}
