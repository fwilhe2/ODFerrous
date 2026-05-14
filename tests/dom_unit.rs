use odferrous::dom::DocumentModel;

#[test]
fn dom_roundtrip_basic() {
    let mut m = DocumentModel::new();
    m.headings.push("Title & More".to_string());
    m.paragraphs
        .push("A paragraph <with> entities & text".to_string());

    let xml = m.to_content_xml();
    let parsed = DocumentModel::from_content_xml(&xml).expect("parse");
    assert_eq!(m, parsed);
}

#[test]
fn dom_empty_roundtrip() {
    let m = DocumentModel::new();
    let xml = m.to_content_xml();
    let parsed = DocumentModel::from_content_xml(&xml).expect("parse empty");
    assert_eq!(m, parsed);
}
