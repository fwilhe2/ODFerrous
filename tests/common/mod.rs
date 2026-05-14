use quick_xml::events::Event;
use quick_xml::Reader;
use std::io::Read;
use std::path::Path;

/// Validate basic well-formedness of XML data.
pub fn validate_xml_well_formed(data: &[u8]) -> Result<(), String> {
    let mut r = Reader::from_reader(data);
    let mut buf = Vec::new();
    loop {
        match r.read_event_into(&mut buf) {
            Ok(Event::Eof) => break,
            Ok(_) => (),
            Err(e) => return Err(format!("XML error: {}", e)),
        }
        buf.clear();
    }
    Ok(())
}

/// Validate output path: if it's a zip, extract content.xml and validate; if flat, validate whole file.
pub fn validate_output(path: &Path) -> Result<(), String> {
    if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
        match ext.to_lowercase().as_str() {
            "odt" | "ods" | "ott" => {
                let data = std::fs::read(path).map_err(|e| e.to_string())?;
                let mut c = std::io::Cursor::new(data);
                let mut archive = zip::ZipArchive::new(&mut c).map_err(|e| e.to_string())?;
                let mut f = archive.by_name("content.xml").map_err(|e| e.to_string())?;
                let mut buf = Vec::new();
                f.read_to_end(&mut buf).map_err(|e| e.to_string())?;
                return validate_xml_well_formed(&buf);
            }
            "fodt" | "fods" | "xml" => {
                let data = std::fs::read(path).map_err(|e| e.to_string())?;
                return validate_xml_well_formed(&data);
            }
            _ => {}
        }
    }
    Err("Unknown output type".into())
}
