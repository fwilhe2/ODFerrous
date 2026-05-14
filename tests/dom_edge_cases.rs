use odferrous::dom::DocumentModel;

#[test]
fn malformed_xml_returns_err() {
    // invalid utf8
    let bad: &[u8] = &[0xff, 0xff, 0xff];
    let res = DocumentModel::from_content_xml(bad);
    assert!(res.is_err(), "expected error for invalid utf8 bytes");
}

#[test]
fn malformed_structure_returns_err() {
    // malformed XML (missing closing)
    let xml =
        b"<?xml version='1.0'?><office:document-content><office:body><office:text><text:p>Unclosed";
    let res = DocumentModel::from_content_xml(xml);
    assert!(res.is_err(), "expected parse error for malformed XML");
}

#[test]
fn large_content_roundtrip() {
    let mut m = DocumentModel::new();
    // create ~1MB payload: 1000 paragraphs of 1024 bytes
    let base = "x".repeat(1024);
    for i in 0..1000 {
        m.paragraphs.push(format!("{}-{}", i, base));
    }

    let xml = m.to_content_xml();
    let parsed = DocumentModel::from_content_xml(&xml).expect("large parse");
    assert_eq!(m, parsed);
}
