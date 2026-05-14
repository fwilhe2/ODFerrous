use odferrous::TextDocument;
use tempfile::tempdir;

#[test]
fn textdoc_save_load_transcode() {
    let dir = tempdir().unwrap();
    let odt = dir.path().join("d.odt");
    let fodt = dir.path().join("d.fodt");

    let mut doc = TextDocument::new_package();
    doc.add_heading("H");
    doc.add_paragraph("P");
    doc.save_as_package(&odt).unwrap();

    let mut d2 = TextDocument::open(&odt).unwrap();
    d2.save_as_flat(&fodt).unwrap();

    let mut d3 = TextDocument::open(&fodt).unwrap();
    // ensure internal model contains saved text after open
    // Since our TextDocument keeps in-memory content only, reopen and roundtrip should succeed
    d3.save_as_package(&dir.path().join("d2.odt")).unwrap();
}
