use crate::responses::{generate_pdf_response, generate_split_pdfs_response};
use crate::types::{GeneratePdfResponse, GenerateSplitPdfsResponse};
use poentaro::{NoteLabels, ResolvedPartVisibility};

use poentaro::fonts::FontBytesByFamily;

#[test]
fn reference_poentaro_generates_pdf() {
    let fonts = FontBytesByFamily::embedded();
    for path in super::demo_file_paths() {
        let source = super::read_demo_file(&path);
        let resp = generate_pdf_response(
            &source,
            &ResolvedPartVisibility::default(),
            &fonts,
            NoteLabels::Degrees,
        );
        match resp {
            GeneratePdfResponse::Ok { pdf } => {
                assert!(pdf.len() > 4);
                assert_eq!(&pdf[0..4], b"%PDF");
            }
            GeneratePdfResponse::Err { diagnostics } => {
                panic!(
                    "{path:?} failed in wasm pdf path: {}",
                    diagnostics[0].message
                );
            }
        }
    }
}

#[test]
fn reference_poentaro_generates_split_pdf_zip() {
    use std::io::Read;
    use zip::ZipArchive;

    let fonts = FontBytesByFamily::embedded();
    for path in super::demo_file_paths() {
        let source = super::read_demo_file(&path);
        let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("demo");
        let resp = generate_split_pdfs_response(&source, stem, None, &fonts, NoteLabels::Degrees);
        match resp {
            GenerateSplitPdfsResponse::Ok { zip } => {
                assert!(zip.len() > 4);
                assert_eq!(&zip[0..2], b"PK");
                let cursor = std::io::Cursor::new(zip);
                let mut archive = ZipArchive::new(cursor).unwrap();
                assert!(!archive.is_empty());
                for i in 0..archive.len() {
                    let mut file = archive.by_index(i).unwrap();
                    let name = file.name().to_string();
                    assert!(
                        name.starts_with(&format!("{stem} - ")) && name.ends_with(".pdf"),
                        "unexpected zip entry: {name}"
                    );
                    let mut buf = [0u8; 4];
                    file.read_exact(&mut buf).unwrap();
                    assert_eq!(&buf, b"%PDF");
                }
            }
            GenerateSplitPdfsResponse::Err { diagnostics } => {
                panic!(
                    "{path:?} failed in wasm split pdf path: {}",
                    diagnostics[0].message
                );
            }
        }
    }
}
