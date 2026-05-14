use odferrous::storage::{FlatFile, Storage, ZipPackage};
use std::path::Path;
use std::sync::{Arc, Barrier};
use std::thread;
use tempfile::tempdir;

#[test]
fn flatfile_large_write_read() {
    let mut f = FlatFile::new_empty();
    // payload ~2MB
    let payload = vec![b'a'; 2 * 1024 * 1024];
    f.write_entry("content.xml", &payload).expect("write");
    let td = tempdir().expect("tempdir");
    let out = td.path().join("flatfile.fodt");
    Box::new(f).persist(&out).expect("persist");

    // non-existent path for zip open should error
    assert!(ZipPackage::open_from_path(Path::new("/does/not/exist")).is_err());

    // re-open from persisted file and verify size
    let mut f2 = FlatFile::open_from_path(&out).unwrap();
    let data = f2.read_entry("content.xml").unwrap();
    assert_eq!(data.len(), payload.len());
}

#[test]
fn concurrent_in_memory_zip_creation() {
    // create several in-memory zip packages in parallel
    let n = 8;
    let barrier = Arc::new(Barrier::new(n));
    let td = tempdir().expect("tempdir");
    let base = td.path().to_path_buf();
    let mut handles = vec![];
    for t in 0..n {
        let c = barrier.clone();
        let base = base.clone();
        handles.push(thread::spawn(move || {
            let mut z = ZipPackage::new_empty();
            let content = format!("thread-{}-payload", t);
            z.write_entry("content.xml", content.as_bytes()).unwrap();
            c.wait();
            let out = base.join(format!("pkg-{}.odt", t));
            Box::new(z).persist(&out).unwrap();
        }));
    }

    for h in handles {
        h.join().expect("thread join");
    }
}
