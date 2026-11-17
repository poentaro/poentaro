//! The `file_versions` SQL run for real against SQLite: recording, listing,
//! pruning, and the cascade when a file row is deleted.
#![allow(clippy::disallowed_macros)]
#![cfg(not(target_arch = "wasm32"))]

mod support;

use live_share_worker::file_history::{rebuild, reverse_patch, VersionRow};
use sqlx::{Executor, Row, SqliteConnection};
use support::*;

const FILE_VERSIONS: &str =
    include_str!("../../../live-share-worker/migrations/0008_file_versions.sql");
const UPDATE_FILE_CONTENT: &str = include_str!("../queries/update_file_content.sql");
const INSERT_FILE_VERSION: &str = include_str!("../queries/insert_file_version.sql");
const ADVANCE_FILE_VERSION_LINK: &str = include_str!("../queries/advance_file_version_link.sql");
const LIST_FILE_VERSIONS: &str = include_str!("../queries/list_file_versions.sql");
const PRUNE_FILE_VERSIONS: &str = include_str!("../queries/prune_file_versions.sql");
const DELETE_FILES_OF_USERS: &str = include_str!("../queries/delete_files_of_users.sql");

async fn setup() -> Result<SqliteConnection, sqlx::Error> {
    let mut connection = connect().await?;
    migrate_up_to_file_shares(&mut connection).await?;
    connection.execute(FILE_VERSIONS).await?;
    add_user(&mut connection, "u1").await?;
    add_user(&mut connection, "u2").await?;
    add_file(&mut connection, "f1", "u1", "a.poentaro", "v0\n").await?;
    Ok(connection)
}

/// Saves `content` the way `handlers::files::update_file_content` does: the
/// CAS update, then the version row and the prune.
async fn save(
    connection: &mut SqliteConnection,
    content: &str,
    keep: i64,
) -> Result<(), sqlx::Error> {
    let head = sqlx::query("SELECT content, revision, updated_at FROM files WHERE id = 'f1'")
        .fetch_one(&mut *connection)
        .await?;
    let previous: String = head.try_get("content")?;
    let revision: i64 = head.try_get("revision")?;
    let updated = sqlx::query(UPDATE_FILE_CONTENT)
        .bind(content)
        .bind(revision + 10)
        .bind("f1")
        .bind("u1")
        .bind(revision)
        .execute(&mut *connection)
        .await?;
    assert_eq!(updated.rows_affected(), 1);
    if previous == content {
        sqlx::query(ADVANCE_FILE_VERSION_LINK)
            .bind("f1")
            .bind(revision)
            .bind(revision + 1)
            .execute(&mut *connection)
            .await?;
        return Ok(());
    }
    sqlx::query(INSERT_FILE_VERSION)
        .bind("f1")
        .bind(revision)
        .bind(revision + 1)
        .bind(reverse_patch(content, &previous))
        .bind(head.try_get::<i64, _>("updated_at")?)
        .execute(&mut *connection)
        .await?;
    sqlx::query(PRUNE_FILE_VERSIONS)
        .bind("f1")
        .bind(keep)
        .execute(&mut *connection)
        .await?;
    Ok(())
}

async fn versions(
    connection: &mut SqliteConnection,
    owner: &str,
) -> Result<Vec<VersionRow>, sqlx::Error> {
    sqlx::query(LIST_FILE_VERSIONS)
        .bind("f1")
        .bind(owner)
        .fetch_all(connection)
        .await?
        .iter()
        .map(|row| {
            Ok(VersionRow {
                revision: row.try_get("revision")?,
                newer_revision: row.try_get("newer_revision")?,
                reverse_patch: row.try_get("reverse_patch")?,
                saved_at: row.try_get("saved_at")?,
            })
        })
        .collect()
}

#[tokio::test]
async fn each_accepted_save_records_a_version_that_rebuilds() -> Result<(), sqlx::Error> {
    let mut connection = setup().await?;
    save(&mut connection, "v1\n", 100).await?;
    save(&mut connection, "v1\n", 100).await?;
    save(&mut connection, "v3\n", 100).await?;

    let rows = versions(&mut connection, "u1").await?;

    let revisions: Vec<(i64, i64)> = rows
        .iter()
        .map(|row| (row.revision, row.newer_revision))
        .collect();
    assert_eq!(revisions, vec![(2, 3), (0, 2)]);
    assert_eq!(rows[0].saved_at, 11);
    assert_eq!(rebuild("v3\n", 3, &rows, 0), Some(Ok("v0\n".to_string())));
    assert_eq!(rebuild("v3\n", 3, &rows, 2), Some(Ok("v1\n".to_string())));
    // Another user sees none of it.
    assert!(versions(&mut connection, "u2").await?.is_empty());
    Ok(())
}

#[tokio::test]
async fn only_the_newest_versions_are_kept() -> Result<(), sqlx::Error> {
    let mut connection = setup().await?;
    for content in ["a\n", "b\n", "c\n", "d\n"] {
        save(&mut connection, content, 2).await?;
    }

    let rows = versions(&mut connection, "u1").await?;

    let revisions: Vec<i64> = rows.iter().map(|row| row.revision).collect();
    assert_eq!(revisions, vec![3, 2]);
    assert_eq!(rebuild("d\n", 4, &rows, 2), Some(Ok("b\n".to_string())));
    Ok(())
}

#[tokio::test]
async fn deleting_a_file_purges_its_history() -> Result<(), sqlx::Error> {
    let mut connection = setup().await?;
    save(&mut connection, "v1\n", 100).await?;

    sqlx::query(DELETE_FILES_OF_USERS)
        .bind(r#"["u1"]"#)
        .execute(&mut connection)
        .await?;

    let left: i64 = sqlx::query("SELECT COUNT(*) AS n FROM file_versions")
        .fetch_one(&mut connection)
        .await?
        .try_get("n")?;
    assert_eq!(left, 0);
    Ok(())
}
