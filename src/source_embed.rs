use base64::Engine;

const OPEN_TAG: &str = r#"<metadata id="poentaro-source">"#;
/// What SVGs exported before the rename to Poentaro carry instead.
const LEGACY_OPEN_TAG: &str = r#"<metadata id="jianpu-source">"#;
const CLOSE_TAG: &str = "</metadata>";

/// Recovers the `.poentaro` source embedded by the serializer (see
/// `serializer::serialize_doc`) inside an SVG's `<metadata id="poentaro-source">`
/// tag (or the legacy `jianpu-source` one). Returns `None` if the tag is absent or its contents aren't valid
/// base64/UTF-8 — e.g. a hand-edited or third-party SVG.
pub fn extract_embedded_source(svg: &str) -> Option<String> {
    let start = [OPEN_TAG, LEGACY_OPEN_TAG]
        .iter()
        .find_map(|tag| Some(svg.find(tag)? + tag.len()))?;
    let end = start + svg[start..].find(CLOSE_TAG)?;
    let decoded = base64::engine::general_purpose::STANDARD
        .decode(&svg[start..end])
        .ok()?;
    String::from_utf8(decoded).ok()
}

const PDF_INFO_KEY: &str = "/PoentaroSource (";
/// What PDFs exported before the rename to Poentaro carry instead.
const LEGACY_PDF_INFO_KEY: &str = "/JianpuSource (";

/// Recovers the `.poentaro` source embedded by `pdf::write_pdf` inside the
/// PDF's `/Info` dictionary under a custom `/PoentaroSource` key (or the
/// legacy `/JianpuSource` one). `usvg` (which
/// parses the page SVGs before PDF conversion) strips `<metadata>` tags, so
/// PDF export can't reuse the SVG embedding approach — see
/// `extract_embedded_source` for that one. Returns `None` if the key is
/// absent or its contents aren't valid base64/UTF-8.
pub fn extract_embedded_source_from_pdf(pdf: &[u8]) -> Option<String> {
    let pdf = String::from_utf8_lossy(pdf);
    let start = [PDF_INFO_KEY, LEGACY_PDF_INFO_KEY]
        .iter()
        .find_map(|key| Some(pdf.find(key)? + key.len()))?;
    let end = start + pdf[start..].find(')')?;
    let decoded = base64::engine::general_purpose::STANDARD
        .decode(&pdf[start..end])
        .ok()?;
    String::from_utf8(decoded).ok()
}

#[cfg(test)]
mod tests;
