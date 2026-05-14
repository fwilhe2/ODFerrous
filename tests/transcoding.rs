use odferrous::TextDocument;
use tempfile::tempdir;
mod common;
use std::io::Read;

#[test]
fn odt_to_fodt_to_odt_transcode() {
    let dir = tempdir().unwrap();
    let p1 = dir.path().join("a.odt");
    let p2 = dir.path().join("a.fodt");
    let p3 = dir.path().join("a2.odt");

    // Create package doc
    let mut doc = TextDocument::new_package();
    doc.add_heading("T1");
    doc.add_paragraph("Hello world");
    doc.save_as_package(&p1).expect("save package");

    // Transcode to flat
    let mut d2 = TextDocument::open(&p1).expect("open odt");
    d2.save_as_flat(&p2).expect("save flat");

    // Validate flat
    common::validate_output(&p2).expect("flat valid");

    // Transcode back to package
    let mut d3 = TextDocument::open(&p2).expect("open fodt");
    d3.save_as_package(&p3).expect("save second package");

    // Validate package
    common::validate_output(&p3).expect("package valid");

    // Compare content.xml bytes
    let c1 = {
        let mut t = std::io::Cursor::new(std::fs::read(&p1).unwrap());
        let mut a = zip::ZipArchive::new(&mut t).unwrap();
        let mut f = a.by_name("content.xml").unwrap();
        let mut b = Vec::new();
        f.read_to_end(&mut b).unwrap();
        b
    };
    let c3 = {
        let mut t = std::io::Cursor::new(std::fs::read(&p3).unwrap());
        let mut a = zip::ZipArchive::new(&mut t).unwrap();
        let mut f = a.by_name("content.xml").unwrap();
        let mut b = Vec::new();
        f.read_to_end(&mut b).unwrap();
        b
    };

    assert_eq!(
        c1, c3,
        "content.xml should roundtrip between package formats"
    );
}
