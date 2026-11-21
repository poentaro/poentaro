//! Conversion of the file store's `FileText` maps to and from WIT.
use super::*;
use poentaro::file_tree as tree;
use std::collections::BTreeMap;

pub(super) fn text_entries(map: BTreeMap<String, tree::FileText>) -> Vec<FileTextEntry> {
    map.into_iter()
        .map(|(key, value)| FileTextEntry {
            key,
            value: match value {
                tree::FileText::Loaded(text) => FileText::Loaded(text),
                tree::FileText::NotLoaded => FileText::NotLoaded,
            },
        })
        .collect()
}

pub(super) fn text_map_of(entries: Vec<FileTextEntry>) -> BTreeMap<String, tree::FileText> {
    entries
        .into_iter()
        .map(|entry| {
            let text = match entry.value {
                FileText::Loaded(text) => tree::FileText::Loaded(text),
                FileText::NotLoaded => tree::FileText::NotLoaded,
            };
            (entry.key, text)
        })
        .collect()
}
