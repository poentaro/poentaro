use crate::diagnostics::diagnostic_from_error;
use crate::opened_document::open_checked;
use crate::types::{
    ListSymbolsResponse, OccurrenceRoleOut, RenameSymbolResponse, SpanOut, SymbolKindOut,
    SymbolOccurrenceOut, SymbolOut, TextEditOut,
};
use poentaro::error::Span;
use poentaro::parser::parts_parser::InstrumentInfo;
use poentaro::symbols::{
    collect_symbols, rename_edits_in_text, OccurrenceRole, Symbol, SymbolKind,
};

fn symbol_kind_to_out(kind: SymbolKind) -> SymbolKindOut {
    match kind {
        SymbolKind::Abbreviation => SymbolKindOut::Abbreviation,
        SymbolKind::SectionLabel => SymbolKindOut::SectionLabel,
    }
}

fn symbol_kind_from_out(kind: SymbolKindOut) -> SymbolKind {
    match kind {
        SymbolKindOut::Abbreviation => SymbolKind::Abbreviation,
        SymbolKindOut::SectionLabel => SymbolKind::SectionLabel,
    }
}

fn occurrence_role_to_out(role: OccurrenceRole) -> OccurrenceRoleOut {
    match role {
        OccurrenceRole::Declaration => OccurrenceRoleOut::Declaration,
        OccurrenceRole::Reference => OccurrenceRoleOut::Reference,
    }
}

fn span_to_out(span: Span) -> SpanOut {
    SpanOut {
        start: span.start,
        end: span.end,
    }
}

fn symbol_to_out(symbol: Symbol) -> SymbolOut {
    SymbolOut {
        name: symbol.name,
        kind: symbol_kind_to_out(symbol.kind),
        occurrences: symbol
            .occurrences
            .into_iter()
            .map(|occurrence| SymbolOccurrenceOut {
                span: span_to_out(occurrence.span),
                hit_span: span_to_out(occurrence.hit_span),
                role: occurrence_role_to_out(occurrence.role),
            })
            .collect(),
    }
}

pub(crate) fn list_symbols_response(
    source: &str,
    instruments: &[InstrumentInfo],
) -> ListSymbolsResponse {
    if let Err(diagnostics) = open_checked(source, instruments) {
        return ListSymbolsResponse::Err { diagnostics };
    }
    match poentaro::parser::parse(source, "input.poentaro", instruments) {
        Ok(parsed) => ListSymbolsResponse::Ok {
            symbols: collect_symbols(&parsed)
                .into_iter()
                .map(symbol_to_out)
                .collect(),
        },
        Err(error) => ListSymbolsResponse::Err {
            diagnostics: vec![diagnostic_from_error(&error)],
        },
    }
}

pub(crate) fn rename_symbol_response(
    source: &str,
    kind: SymbolKindOut,
    old_name: &str,
    new_name: &str,
    instruments: &[InstrumentInfo],
) -> RenameSymbolResponse {
    if let Err(diagnostics) = open_checked(source, instruments) {
        return RenameSymbolResponse::Err { diagnostics };
    }
    let parsed = match poentaro::parser::parse(source, "input.poentaro", instruments) {
        Ok(parsed) => parsed,
        Err(error) => {
            return RenameSymbolResponse::Err {
                diagnostics: vec![diagnostic_from_error(&error)],
            }
        }
    };
    let edits = rename_edits_in_text(
        source,
        &parsed,
        symbol_kind_from_out(kind),
        old_name,
        new_name,
    );
    RenameSymbolResponse::Ok {
        edits: edits
            .into_iter()
            .map(|edit| TextEditOut {
                span: span_to_out(edit.span),
                replacement: edit.replacement,
            })
            .collect(),
    }
}
