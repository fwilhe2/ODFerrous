use odferrous::storage::Storage;
use odferrous::storage::{detect_format, FlatFile, ZipPackage};
use tempfile::tempdir;

#[test]
fn zip_package_write_and_read() {
    let dir = tempdir().unwrap();
    let p = dir.path().join("t.odt");

    let mut pkg = ZipPackage::new_empty();
    Storage::write_entry(&mut pkg, "content.xml", b"<root/>").unwrap();
    Box::new(pkg).persist(&p).unwrap();

    // reopen
    let mut pkg2 = ZipPackage::open_from_path(&p).unwrap();
    let data = Storage::read_entry(&mut pkg2, "content.xml").unwrap();
    assert_eq!(data, b"<root/>".to_vec());
}

#[test]
fn flatfile_write_and_read() {
    let dir = tempdir().unwrap();
    let p = dir.path().join("t.fodt");
    let mut f = FlatFile::new_empty();
    Storage::write_entry(&mut f, "content.xml", b"<root/>").unwrap();
    Box::new(f).persist(&p).unwrap();

    let mut f2 = FlatFile::open_from_path(&p).unwrap();
    let data = Storage::read_entry(&mut f2, "content.xml").unwrap();
    assert_eq!(data, b"<root/>".to_vec());
}

#[test]
fn detect_format_by_extension() {
    let p1 = std::path::Path::new("doc.odt");
    let p2 = std::path::Path::new("doc.fodt");
    assert!(matches!(
        detect_format(p1).unwrap(),
        odferrous::storage::DetectedFormat::Zip
    ));
    assert!(matches!(
        detect_format(p2).unwrap(),
        odferrous::storage::DetectedFormat::Flat
    ));
}
