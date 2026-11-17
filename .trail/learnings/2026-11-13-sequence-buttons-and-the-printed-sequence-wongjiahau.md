---
title: Sequence buttons and the printed Sequence line share one label format
author: Wongjiahau
date: 2026-11-13
branch: fix/sequence-button-with-label
cost: { minutes: 30, tokens: 121102 }
id: l-59787c06
scope: repo
status: active
src: agent
touches:
  - crates/poentaro-wasm/src/responses.rs
  - crates/poentaro-wasm/src/tests_parts_and_measures.rs
  - src/ast/grouped.rs
  - src/ast/grouped_sequence.rs
  - src/coordinate_resolver/directive_line_conversion.rs
  - web/e2e/features/sequence-jump-toolbar-line-choice.feature
  - web/e2e/features/steps/sequence-jump-toolbar-line-choice.steps.ts
  - web/e2e/features/play-selection-follows-sequence-entry.feature
  - web/e2e/features/steps/play-selection-follows-sequence-entry.steps.ts
  - web/src/hooks/useAppSelectionAndNavigation.ts
---

## Decided
The web app's Sequence buttons and the SVG/PDF "Sequence: ..." line build an entry's suffixes (WITH line choices, WITHOUT/ONLY parts) from the same Rust formatters. They had drifted: the buttons ignored WITH, so two different verses both read "Verse". As part of this, the button label adopted the printed line's spacing ("Y (-b)", not "Y(-b)").

## Watch out
Any new sequence-entry modifier must be added to the shared formatter, not to one of the two call sites, or the buttons and the printout disagree again. Keep the label logic in Rust; the toolbar only renders the string.

In the code: `SequenceSpan::display_label`, `line_choices_suffix`, `part_filter_suffix` in src/ast/grouped_sequence.rs; used by crates/poentaro-wasm/src/responses.rs (button label) and `sequence_line_content` in src/coordinate_resolver/directive_line_conversion.rs.
