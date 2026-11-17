use std::collections::HashMap;

use crate::ast::parsed::{AlternativeLine, PartDecl};
use crate::error::{RecoverableError, Span};

use super::attribution::KeyedLine;
use super::{GroupContext, SourceLine};

/// One entry per declaration, aligned with `declarations`: the line the
/// declaration's `[Key]` addressed in this measure group, if any.
pub(super) type KeyMap = Vec<Option<SourceLine>>;

pub(super) fn filter_keyed_into_key_map(
    keyed: Vec<KeyedLine>,
    declarations: &[PartDecl],
    context: &GroupContext,
    recoverable_error: &mut Option<RecoverableError>,
) -> KeyMap {
    let mut key_map: KeyMap = declarations.iter().map(|_| None).collect();
    let mut lines_so_far: HashMap<String, u32> = HashMap::new();

    for line in keyed {
        // A part's Nth line in the group goes to its alternative line `Key.N`
        // (see `parser::alternative_lines`).
        let number = {
            let count = lines_so_far.entry(line.key.clone()).or_default();
            *count += 1;
            *count
        };
        let abbreviation = if number == 1 {
            line.key.clone()
        } else {
            AlternativeLine {
                base: line.key.clone(),
                number,
            }
            .abbreviation()
        };
        let Some(declaration_index) = declarations
            .iter()
            .position(|declaration| declaration.abbreviation == abbreviation)
        else {
            recoverable_error.get_or_insert_with(|| {
                RecoverableError::part_key_unknown(line.key_prefix_span, &line.key)
            });
            continue;
        };
        let source_line = SourceLine {
            content: line.content,
            offset: line.content_offset,
            is_implicit_fill: false,
        };
        let Some(slot) = key_map.get_mut(declaration_index) else {
            continue;
        };
        if slot.is_some() {
            let line_span = Span::new(
                context.base_offset + source_line.offset,
                context.base_offset + source_line.offset + 1,
            );
            recoverable_error.get_or_insert_with(|| {
                RecoverableError::general(
                    line_span,
                    format!("part [{}] has 2 lines but only 1 slot(s)", line.key),
                )
            });
        } else {
            *slot = Some(source_line);
        }
    }

    key_map
}
