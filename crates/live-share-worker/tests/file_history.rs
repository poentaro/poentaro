//! Property tests for `file_history`: they target our inputs (CJK, `\r\n`,
//! no trailing newline, real `.poentaro` files) and our chain glue, not the
//! diff library's algorithm.
#![allow(clippy::disallowed_macros)]
#![cfg(not(target_arch = "wasm32"))]

use live_share_worker::file_history::{
    apply_patch, reachable_versions, rebuild, reverse_patch, VersionRow,
};
use proptest::prelude::*;

const REAL_FILES: &[&str] = &[
    include_str!("../../../demo/03-slurs-and-ties.poentaro"),
    include_str!("../../../demo/05-lyrics.poentaro"),
    include_str!("../../../demo/10-multi-part.poentaro"),
    include_str!("../../../scores/Air on G String.poentaro"),
    include_str!("../../../scores/Minuet in G.poentaro"),
];

/// One line, biased toward what diff libraries get wrong.
fn line() -> impl Strategy<Value = String> {
    prop_oneof![
        Just(String::new()),
        Just("1 2 3 4 |".to_string()),
        Just("1 2 3 4 |".to_string()),
        Just("哈利路亚 主耶稣".to_string()),
        Just("e\u{301} n\u{303} combining".to_string()),
        Just(" \\ No newline at end of file".to_string()),
        Just("--- a".to_string()),
        Just("+++ b".to_string()),
        Just("@@ -1 +1 @@".to_string()),
        "[ -~]{0,12}",
        "\\PC{0,8}",
    ]
}

fn line_break() -> impl Strategy<Value = &'static str> {
    prop_oneof![Just("\n"), Just("\r\n")]
}

/// Lines joined by mixed `\n` / `\r\n`, with or without a final break.
fn synthetic_text() -> impl Strategy<Value = String> {
    (
        prop::collection::vec((line(), line_break()), 0..12),
        any::<bool>(),
    )
        .prop_map(|(lines, trailing)| {
            let mut text: String = lines
                .iter()
                .flat_map(|(line, brk)| [line.as_str(), brk])
                .collect();
            if !trailing {
                let trimmed = text.trim_end_matches(['\n', '\r']).len();
                text.truncate(trimmed);
            }
            text
        })
}

/// A real file with a few lines replaced, removed or duplicated.
fn mutated_real_file() -> impl Strategy<Value = String> {
    (
        prop::sample::select(REAL_FILES),
        prop::collection::vec((any::<prop::sample::Index>(), 0..3u8, line()), 0..4),
    )
        .prop_map(|(file, edits)| {
            let mut lines: Vec<String> = file.split('\n').map(str::to_string).collect();
            for (index, edit, replacement) in edits {
                let at = index.index(lines.len());
                match edit {
                    0 => {
                        if let Some(line) = lines.get_mut(at) {
                            *line = replacement;
                        }
                    }
                    1 if lines.len() > 1 => {
                        lines.remove(at);
                    }
                    _ => {
                        if let Some(line) = lines.get(at).cloned() {
                            lines.insert(at, line);
                        }
                    }
                }
            }
            lines.join("\n")
        })
}

fn text() -> impl Strategy<Value = String> {
    prop_oneof![synthetic_text(), mutated_real_file(), "\\PC{0,40}"]
}

/// A file's head plus the version rows a series of saves leaves, the way the
/// worker records them (a no-op save advances the newest link).
struct History {
    head: String,
    head_revision: i64,
    rows: Vec<VersionRow>,
    saved: Vec<(i64, String)>,
}

impl History {
    fn new(content: &str) -> Self {
        History {
            head: content.to_string(),
            head_revision: 0,
            rows: Vec::new(),
            saved: vec![(0, content.to_string())],
        }
    }

    fn save(&mut self, content: &str) {
        let new_revision = self.head_revision + 1;
        if content == self.head {
            self.rows
                .iter_mut()
                .filter(|row| row.newer_revision == self.head_revision)
                .for_each(|row| row.newer_revision = new_revision);
        } else {
            self.rows.push(VersionRow {
                revision: self.head_revision,
                newer_revision: new_revision,
                reverse_patch: reverse_patch(content, &self.head),
                saved_at: self.head_revision,
            });
        }
        self.head = content.to_string();
        self.head_revision = new_revision;
        self.saved.push((new_revision, content.to_string()));
    }

    /// Keeps only the newest `keep` rows, like `prune_file_versions.sql`.
    fn prune(&mut self, keep: usize) {
        self.rows.sort_by(|a, b| b.revision.cmp(&a.revision));
        self.rows.truncate(keep);
    }

    fn assert_every_version_rebuilds(&self) -> Result<(), TestCaseError> {
        for row in reachable_versions(self.head_revision, &self.rows) {
            let expected = self
                .saved
                .iter()
                .find(|(revision, _)| *revision == row.revision)
                .map(|(_, content)| content);
            let rebuilt = rebuild(&self.head, self.head_revision, &self.rows, row.revision)
                .ok_or_else(|| TestCaseError::fail("a reachable version did not rebuild"))?
                .map_err(|error| TestCaseError::fail(error.message))?;
            prop_assert_eq!(Some(&rebuilt), expected);
        }
        Ok(())
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    #[test]
    fn a_patch_round_trips(a in text(), b in text()) {
        let patch = diffy::create_patch(&a, &b).to_string();
        prop_assert_eq!(apply_patch(&patch, &a).map_err(|e| TestCaseError::fail(e.message))?, b);
    }

    #[test]
    fn the_reverse_patch_rebuilds_the_older_text(older in text(), newer in text()) {
        let patch = reverse_patch(&newer, &older);
        prop_assert_eq!(apply_patch(&patch, &newer).map_err(|e| TestCaseError::fail(e.message))?, older);
    }

    #[test]
    fn every_retained_version_rebuilds_to_what_was_saved(
        first in text(),
        saves in prop::collection::vec((text(), any::<bool>()), 0..12),
    ) {
        let mut history = History::new(&first);
        for (content, repeat) in saves {
            history.save(&content);
            if repeat {
                history.save(&content);
            }
        }
        // Every save that changed the text is reachable.
        let changes = history.saved.windows(2).filter(|pair| pair[0].1 != pair[1].1).count();
        prop_assert_eq!(reachable_versions(history.head_revision, &history.rows).len(), changes);
        history.assert_every_version_rebuilds()?;
    }

    #[test]
    fn every_version_left_after_a_prune_still_rebuilds(
        first in text(),
        saves in prop::collection::vec(text(), 0..12),
        keep in 0usize..6,
    ) {
        let mut history = History::new(&first);
        saves.iter().for_each(|content| history.save(content));
        history.prune(keep);
        history.assert_every_version_rebuilds()?;
    }

    #[test]
    fn restoring_a_version_makes_it_the_head_and_grows_history_by_one(
        first in text(),
        saves in prop::collection::vec(text(), 1..8),
        pick in any::<prop::sample::Index>(),
    ) {
        let mut history = History::new(&first);
        saves.iter().for_each(|content| history.save(content));
        let versions: Vec<i64> = reachable_versions(history.head_revision, &history.rows)
            .iter()
            .map(|row| row.revision)
            .collect();
        prop_assume!(!versions.is_empty());
        let chosen = *pick.get(&versions);
        let restored = rebuild(&history.head, history.head_revision, &history.rows, chosen)
            .ok_or_else(|| TestCaseError::fail("a reachable version did not rebuild"))?
            .map_err(|error| TestCaseError::fail(error.message))?;
        prop_assume!(restored != history.head);

        history.save(&restored);

        prop_assert_eq!(&history.head, &restored);
        prop_assert_eq!(
            reachable_versions(history.head_revision, &history.rows).len(),
            versions.len() + 1
        );
        history.assert_every_version_rebuilds()?;
    }
}

#[test]
fn a_gap_in_the_chain_hides_older_versions() {
    let row = |revision, newer_revision| VersionRow {
        revision,
        newer_revision,
        reverse_patch: reverse_patch("b\n", "a\n"),
        saved_at: 0,
    };
    // Revision 2 -> 3 failed to record.
    let rows = vec![row(0, 1), row(1, 2), row(3, 4)];

    let reachable: Vec<i64> = reachable_versions(4, &rows)
        .iter()
        .map(|row| row.revision)
        .collect();

    assert_eq!(reachable, vec![3]);
    assert!(rebuild("b\n", 4, &rows, 1).is_none());
}
