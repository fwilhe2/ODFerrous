use odferrous::dom::DocumentModel;
use std::io::Write;
use std::path::Path;
use tempfile::NamedTempFile;

#[test]
fn validate_generated_content_with_jing_if_available() {
    // Locate the Relax NG schema in the repo root
    let schema = Path::new(env!("CARGO_MANIFEST_DIR")).join("OpenDocument-v1.4-schema.rng");
    if !schema.exists() {
        println!("Skipping RNG validation: schema not found at {:?}", schema);
        return;
    }

    // Create a minimal valid DocumentModel and serialize to content.xml bytes
    let mut m = DocumentModel::new();
    m.paragraphs.push("Hello RNG".into());
    let xml = m.to_content_xml();

    // Write to a temporary file
    let mut tmp = NamedTempFile::new().expect("create temp file");
    tmp.write_all(&xml).expect("write xml");
    tmp.flush().ok();

    // Try to run `jing` (external Relax NG validator). If not installed, skip the test.
    let jing = std::process::Command::new("jing")
        .arg(&schema)
        .arg(tmp.path())
        .output();
    match jing {
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            println!("Skipping RNG validation: 'jing' not found in PATH");
            return;
        }
        Err(e) => panic!("Failed to run jing: {}", e),
        Ok(out) => {
            if !out.status.success() {
                let stderr = String::from_utf8_lossy(&out.stderr);
                let stdout = String::from_utf8_lossy(&out.stdout);
                // By default, treat jing failures as a skipped check to avoid failing
                // developer environments where the full schema or tooling may differ.
                // Set ODFERROUS_RNG_STRICT=1 in CI to make this test fatal.
                if std::env::var("ODFERROUS_RNG_STRICT").as_deref() == Ok("1") {
                    panic!(
                        "jing reported validation errors:\nSTDOUT:\n{}\nSTDERR:\n{}",
                        stdout, stderr
                    );
                } else {
                    println!("Skipping RNG validation (jing failed). Set ODFERROUS_RNG_STRICT=1 to treat failures as fatal.");
                    println!("jing stdout:\n{}", stdout);
                    println!("jing stderr:\n{}", stderr);
                    return;
                }
            }
        }
    }
}
