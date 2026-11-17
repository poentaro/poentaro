//! A batch rename (a folder move) is all-or-nothing: the planning rules in
//! `files::plan_batch_rename`, and the real SQL run against SQLite.
#![allow(clippy::disallowed_macros)]

use live_share_worker::files::{plan_batch_rename, BatchRenamePlan, FileNameRow};
use live_share_worker::model::RenameBatchMove;

fn moves(pairs: &[(&str, &str)]) -> Vec<RenameBatchMove> {
    pairs
        .iter()
        .map(|(id, name)| RenameBatchMove {
            id: (*id).to_string(),
            name: (*name).to_string(),
        })
        .collect()
}

fn rows(pairs: &[(&str, &str)]) -> Vec<FileNameRow> {
    pairs
        .iter()
        .map(|(id, name)| FileNameRow {
            id: (*id).to_string(),
            name: (*name).to_string(),
        })
        .collect()
}

#[test]
fn applies_when_every_file_exists_and_no_name_is_taken() {
    let plan = plan_batch_rename(
        &moves(&[("a", "New/a"), ("b", "New/b")]),
        &rows(&[("a", "Old/a"), ("b", "Old/b")]),
    );

    assert_eq!(plan, BatchRenamePlan::Apply);
}

#[test]
fn a_missing_file_fails_the_whole_batch() {
    let plan = plan_batch_rename(
        &moves(&[("a", "New/a"), ("gone", "New/gone")]),
        &rows(&[("a", "Old/a")]),
    );

    assert_eq!(plan, BatchRenamePlan::NotFound);
}

#[test]
fn a_repeat_of_an_applied_batch_is_a_success_not_a_collision() {
    // The first attempt succeeded and only its answer was lost: the files
    // already carry the requested names, which are "taken" by themselves.
    let plan = plan_batch_rename(
        &moves(&[("a", "New/a"), ("b", "New/b")]),
        &rows(&[("a", "New/a"), ("b", "New/b")]),
    );

    assert_eq!(plan, BatchRenamePlan::AlreadyApplied);
}

#[test]
fn a_name_held_by_another_file_is_reported_and_nothing_else() {
    let plan = plan_batch_rename(
        &moves(&[("a", "New/a"), ("b", "New/b")]),
        &rows(&[("a", "Old/a"), ("b", "Old/b"), ("other", "New/b")]),
    );

    assert_eq!(plan, BatchRenamePlan::NamesTaken(vec!["New/b".to_string()]));
}

#[test]
fn a_name_held_by_a_trashed_file_is_taken_too() {
    // Rows carry no trashed flag on purpose: the unique index does not care.
    let plan = plan_batch_rename(
        &moves(&[("a", "New/a")]),
        &rows(&[("a", "Old/a"), ("binned", "New/a")]),
    );

    assert_eq!(plan, BatchRenamePlan::NamesTaken(vec!["New/a".to_string()]));
}

#[test]
fn a_name_held_by_another_file_of_the_batch_is_taken() {
    let plan = plan_batch_rename(
        &moves(&[("a", "Two"), ("b", "Three")]),
        &rows(&[("a", "One"), ("b", "Two")]),
    );

    assert_eq!(plan, BatchRenamePlan::NamesTaken(vec!["Two".to_string()]));
}

#[test]
fn a_file_or_name_listed_twice_is_invalid() {
    let rows = rows(&[("a", "Old/a"), ("b", "Old/b")]);

    assert_eq!(
        plan_batch_rename(&moves(&[("a", "X/1"), ("a", "X/2")]), &rows),
        BatchRenamePlan::Invalid
    );
    assert_eq!(
        plan_batch_rename(&moves(&[("a", "X/1"), ("b", "X/1")]), &rows),
        BatchRenamePlan::Invalid
    );
}

#[cfg(not(target_arch = "wasm32"))]
mod sql {
    use sqlx::sqlite::SqliteConnectOptions;
    use sqlx::{ConnectOptions, Executor, Row};

    const FILES_MIGRATION: &str =
        include_str!("../../../live-share-worker/migrations/0003_files.sql");
    const RENAME_FILES_BATCH: &str = include_str!("../queries/rename_files_batch.sql");

    async fn database_with(files: &[(&str, &str)]) -> Result<sqlx::SqliteConnection, sqlx::Error> {
        let mut connection = SqliteConnectOptions::new()
            .filename(":memory:")
            .foreign_keys(false)
            .connect()
            .await?;
        connection.execute(FILES_MIGRATION).await?;
        for (id, name) in files {
            sqlx::query(
                "INSERT INTO files (id, owner_user_id, name, content, created_at, updated_at) \
                 VALUES (?1, 'owner', ?2, '', 0, 0)",
            )
            .bind(id)
            .bind(name)
            .execute(&mut connection)
            .await?;
        }
        Ok(connection)
    }

    async fn names(connection: &mut sqlx::SqliteConnection) -> Result<Vec<String>, sqlx::Error> {
        sqlx::query("SELECT name FROM files ORDER BY id")
            .fetch_all(connection)
            .await?
            .iter()
            .map(|row| row.try_get("name"))
            .collect()
    }

    async fn run(
        connection: &mut sqlx::SqliteConnection,
        moves_json: &str,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(RENAME_FILES_BATCH)
            .bind(moves_json)
            .bind(5)
            .bind("owner")
            .execute(connection)
            .await
            .map(drop)
    }

    #[tokio::test]
    async fn renames_every_listed_file_in_one_statement() -> Result<(), sqlx::Error> {
        let mut connection =
            database_with(&[("a", "Old/a"), ("b", "Old/b"), ("c", "Keep")]).await?;

        run(
            &mut connection,
            r#"[{"id":"a","name":"New/a"},{"id":"b","name":"New/b"}]"#,
        )
        .await?;

        assert_eq!(names(&mut connection).await?, ["New/a", "New/b", "Keep"]);
        Ok(())
    }

    #[tokio::test]
    async fn a_collision_applies_none_of_the_moves() -> Result<(), sqlx::Error> {
        let mut connection =
            database_with(&[("a", "Old/a"), ("b", "Old/b"), ("c", "New/b")]).await?;

        let outcome = run(
            &mut connection,
            r#"[{"id":"a","name":"New/a"},{"id":"b","name":"New/b"}]"#,
        )
        .await;

        assert!(outcome.is_err(), "the unique index must reject New/b");
        assert_eq!(names(&mut connection).await?, ["Old/a", "Old/b", "New/b"]);
        Ok(())
    }
}
