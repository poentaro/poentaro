//! The folder-share SQL run for real against SQLite (1/2): the migration over
//! existing rows, the one-of constraint, start/stop/reuse and ownership.
#![allow(clippy::disallowed_macros)]
#![cfg(not(target_arch = "wasm32"))]

mod support;

use sqlx::{Executor, Row};
use support::*;

#[tokio::test]
async fn the_migration_keeps_existing_file_shares_and_backfills_their_owner(
) -> Result<(), sqlx::Error> {
    let mut connection = connect().await?;
    migrate_up_to_file_shares(&mut connection).await?;
    add_user(&mut connection, "u1").await?;
    add_user(&mut connection, "u2").await?;
    add_file(&mut connection, "f1", "u1", "Hymns/a.poentaro", "one").await?;
    add_file(&mut connection, "f2", "u2", "b.poentaro", "two").await?;
    sqlx::query(
        "INSERT INTO docs (share_id, owner_user_id, filename, content, ended, created_at, updated_at, external_file_id) \
         VALUES ('share-live', 'u1', 'x', 'x', 0, 11, 11, 'f1'), ('share-stopped', 'u2', 'x', 'x', 1, 12, 13, 'f2')",
    )
    .execute(&mut connection)
    .await?;
    connection.execute(SHARES_POINT_AT_FILES).await?;

    connection.execute(FOLDER_SHARES).await?;

    let rows = sqlx::query(
        "SELECT share_id, owner_user_id, file_id, folder_prefix, ended_at, created_at \
         FROM shares ORDER BY share_id",
    )
    .fetch_all(&mut connection)
    .await?;
    let summary: Vec<MigratedShare> = rows
        .iter()
        .map(|row| {
            Ok(MigratedShare {
                share_id: row.try_get("share_id")?,
                owner_user_id: row.try_get("owner_user_id")?,
                file_id: row.try_get("file_id")?,
                folder_prefix: row.try_get("folder_prefix")?,
                ended_at: row.try_get("ended_at")?,
                created_at: row.try_get("created_at")?,
            })
        })
        .collect::<Result<_, sqlx::Error>>()?;
    let migrated = |share_id: &str, owner: &str, file: &str, ended_at, created_at| MigratedShare {
        share_id: share_id.to_string(),
        owner_user_id: owner.to_string(),
        file_id: file.to_string(),
        folder_prefix: None,
        ended_at,
        created_at,
    };
    assert_eq!(
        summary,
        vec![
            migrated("share-live", "u1", "f1", None, 11),
            migrated("share-stopped", "u2", "f2", Some(13), 12),
        ]
    );
    // The file queries still work on the migrated rows.
    let view = sqlx::query(GET_SHARE_VIEW)
        .bind("share-live")
        .fetch_one(&mut connection)
        .await?;
    assert_eq!(view.try_get::<String, _>("content")?, "one");
    let status = sqlx::query(GET_SHARE_STATUS_BY_FILE)
        .bind("f2")
        .bind("u2")
        .fetch_one(&mut connection)
        .await?;
    assert_eq!(status.try_get::<Option<i64>, _>("ended_at")?, Some(13));
    Ok(())
}

#[tokio::test]
async fn a_share_is_either_a_file_or_a_folder_never_both_or_neither() -> Result<(), sqlx::Error> {
    let mut connection = migrated().await?;
    add_file(&mut connection, "f1", "u1", "a.poentaro", "").await?;

    let both = connection
        .execute("INSERT INTO shares (share_id, owner_user_id, file_id, folder_prefix, created_at) VALUES ('s1','u1','f1','Hymns',0)")
        .await;
    let neither = connection
        .execute("INSERT INTO shares (share_id, owner_user_id, file_id, folder_prefix, created_at) VALUES ('s2','u1',NULL,NULL,0)")
        .await;

    assert!(both.is_err());
    assert!(neither.is_err());
    Ok(())
}

#[tokio::test]
async fn stopping_and_starting_a_folder_again_reuses_its_share_id() -> Result<(), sqlx::Error> {
    let mut connection = migrated().await?;

    let first = start_folder(&mut connection, "id-one", "u1", "Hymns/Advent").await?;
    let stopped = sqlx::query(END_FOLDER_SHARE)
        .bind(5)
        .bind("u1")
        .bind("Hymns/Advent")
        .execute(&mut connection)
        .await?;
    let status = sqlx::query(GET_FOLDER_SHARE_STATUS)
        .bind("u1")
        .bind("Hymns/Advent")
        .fetch_one(&mut connection)
        .await?;
    let again = start_folder(&mut connection, "id-two", "u1", "Hymns/Advent").await?;
    let live = sqlx::query(GET_FOLDER_SHARE_STATUS)
        .bind("u1")
        .bind("Hymns/Advent")
        .fetch_one(&mut connection)
        .await?;

    assert_eq!(first, "id-one");
    assert_eq!(stopped.rows_affected(), 1);
    assert_eq!(status.try_get::<Option<i64>, _>("ended_at")?, Some(5));
    assert_eq!(again, "id-one");
    assert_eq!(live.try_get::<Option<i64>, _>("ended_at")?, None);
    Ok(())
}

#[tokio::test]
async fn only_the_owner_can_stop_or_see_a_folder_share() -> Result<(), sqlx::Error> {
    let mut connection = migrated().await?;
    start_folder(&mut connection, "id-one", "u1", "Hymns").await?;

    let stopped_by_other = sqlx::query(END_FOLDER_SHARE)
        .bind(5)
        .bind("u2")
        .bind("Hymns")
        .execute(&mut connection)
        .await?;
    let status_for_other = sqlx::query(GET_FOLDER_SHARE_STATUS)
        .bind("u2")
        .bind("Hymns")
        .fetch_optional(&mut connection)
        .await?;
    let other_gets_own_link = start_folder(&mut connection, "id-two", "u2", "Hymns").await?;

    assert_eq!(stopped_by_other.rows_affected(), 0);
    assert!(status_for_other.is_none());
    assert_eq!(other_gets_own_link, "id-two");
    Ok(())
}

#[tokio::test]
async fn a_file_share_upsert_records_the_owner_and_reuses_its_id() -> Result<(), sqlx::Error> {
    let mut connection = migrated().await?;
    add_file(&mut connection, "f1", "u1", "a.poentaro", "").await?;
    let upsert = |id: &'static str| {
        sqlx::query(UPSERT_SHARE)
            .bind(id)
            .bind("u1")
            .bind("f1")
            .bind(1)
    };

    let first: String = upsert("id-one")
        .fetch_one(&mut connection)
        .await?
        .try_get("share_id")?;
    sqlx::query(END_SHARE)
        .bind(5)
        .bind("f1")
        .bind("u1")
        .execute(&mut connection)
        .await?;
    let again: String = upsert("id-two")
        .fetch_one(&mut connection)
        .await?
        .try_get("share_id")?;
    let stopped_by_other = sqlx::query(END_SHARE)
        .bind(6)
        .bind("f1")
        .bind("u2")
        .execute(&mut connection)
        .await?;

    assert_eq!((first.as_str(), again.as_str()), ("id-one", "id-one"));
    assert_eq!(stopped_by_other.rows_affected(), 0);
    Ok(())
}
