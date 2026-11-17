#![cfg_attr(test, allow(clippy::disallowed_macros))]

mod component;
mod diagnostics;
mod folder_share_payload;
mod lyric_selection_types;
#[cfg(feature = "wav")]
mod note_selection_trim;
mod note_selection_types;
mod opened_document;
mod part_declarations;
mod responses;
pub mod selection_range;
mod staff_measures;
mod svg_types;
mod svg_types_convert;
mod symbols;
mod types;
#[cfg(any(feature = "wav", feature = "mp3", feature = "pdf", feature = "midi"))]
mod types_export;

/// Combines a `sequence` entry index pair from the wasm boundary (where
/// `Option<RangeInclusive<usize>>` can't cross directly) back into the range
/// [`poentaro::MeasureRangeSelection::sequence_entry_range`] expects.
/// `None` unless both bounds are present, since a partial pair can't name a range.
#[cfg(any(feature = "wav", feature = "midi"))]
pub(crate) fn sequence_entry_range(
    start: Option<usize>,
    end: Option<usize>,
) -> Option<std::ops::RangeInclusive<usize>> {
    match (start, end) {
        (Some(start), Some(end)) => Some(start..=end),
        _ => None,
    }
}

/// Combines a trim-window second pair from the wasm boundary into the
/// [`poentaro::wav::TrimWindow`]
/// [`poentaro::write_wav_for_measure_range_from_source`] expects.
/// `None` unless both bounds are present, since a partial pair can't name a
/// window.
#[cfg(feature = "wav")]
pub(crate) fn trim_window(
    start_s: Option<f64>,
    end_s: Option<f64>,
    next_note_start_s: Option<f64>,
) -> Option<poentaro::wav::TrimWindow> {
    match (start_s, end_s) {
        (Some(start_s), Some(end_s)) => Some(poentaro::wav::TrimWindow {
            start_s,
            end_s,
            next_note_start_s,
        }),
        _ => None,
    }
}

/// Compress a share-link payload with brotli (quality 11).
///
/// The caller is responsible for base64url-encoding the result for use in a URL.
pub(crate) fn compress_share_payload_bytes(payload: &str) -> Vec<u8> {
    let params = brotli::enc::BrotliEncoderParams {
        quality: 11,
        ..Default::default()
    };
    let mut output = Vec::new();
    // Writing to an in-memory `Vec<u8>` cannot produce an I/O error, so any
    // `Err` here is unreachable in practice; ignore it rather than panicking.
    if brotli::BrotliCompress(&mut payload.as_bytes(), &mut output, &params).is_err() {
        return Vec::new();
    }
    output
}

/// Decompress a brotli-compressed share-link payload back into a UTF-8
/// string. Shared the same way as [`compress_share_payload_bytes`] above.
///
/// Returns `None` if `bytes` is not valid brotli, or decompresses to invalid UTF-8.
pub(crate) fn decompress_share_payload_bytes(bytes: &[u8]) -> Option<String> {
    let mut output = Vec::new();
    brotli::BrotliDecompress(&mut &bytes[..], &mut output).ok()?;
    String::from_utf8(output).ok()
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
