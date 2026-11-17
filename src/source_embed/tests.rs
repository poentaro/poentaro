use super::*;

#[test]
fn extracts_base64_encoded_source() {
    let encoded = base64::engine::general_purpose::STANDARD.encode("[Melody] 1 2 3");
    let svg = format!(r#"<svg><metadata id="poentaro-source">{encoded}</metadata></svg>"#);
    assert_eq!(
        extract_embedded_source(&svg).as_deref(),
        Some("[Melody] 1 2 3")
    );
}

#[test]
fn returns_none_when_tag_absent() {
    assert_eq!(extract_embedded_source("<svg></svg>"), None);
}

#[test]
fn returns_none_when_contents_are_not_valid_base64() {
    let svg = r#"<svg><metadata id="poentaro-source">not-valid-base64!!!</metadata></svg>"#;
    assert_eq!(extract_embedded_source(svg), None);
}

#[test]
fn extracts_base64_encoded_source_from_pdf_info_dict() {
    let encoded = base64::engine::general_purpose::STANDARD.encode("[Melody] 1 2 3");
    let pdf = format!("%PDF-1.7\n1 0 obj\n<< /PoentaroSource ({encoded}) >>\nendobj\n");
    assert_eq!(
        extract_embedded_source_from_pdf(pdf.as_bytes()).as_deref(),
        Some("[Melody] 1 2 3")
    );
}

#[test]
fn returns_none_when_pdf_key_absent() {
    assert_eq!(extract_embedded_source_from_pdf(b"%PDF-1.7\n"), None);
}

#[test]
fn returns_none_when_pdf_contents_are_not_valid_base64() {
    let pdf = b"%PDF-1.7\n1 0 obj\n<< /PoentaroSource (not-valid-base64!!!) >>\nendobj\n";
    assert_eq!(extract_embedded_source_from_pdf(pdf), None);
}

#[test]
fn extracts_source_from_an_svg_exported_before_the_rename() {
    let encoded = base64::engine::general_purpose::STANDARD.encode("[Melody] 1 2 3");
    let svg = format!(r#"<svg><metadata id="jianpu-source">{encoded}</metadata></svg>"#);
    assert_eq!(
        extract_embedded_source(&svg).as_deref(),
        Some("[Melody] 1 2 3")
    );
}

#[test]
fn extracts_source_from_a_pdf_exported_before_the_rename() {
    let encoded = base64::engine::general_purpose::STANDARD.encode("[Melody] 1 2 3");
    let pdf = format!("%PDF-1.7\n1 0 obj\n<< /JianpuSource ({encoded}) >>\nendobj\n");
    assert_eq!(
        extract_embedded_source_from_pdf(pdf.as_bytes()).as_deref(),
        Some("[Melody] 1 2 3")
    );
}
