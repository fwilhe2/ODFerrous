use crate::error::Result;
use crate::storage::{detect_format, DetectedFormat, FlatFile, Storage, ZipPackage};
use std::io::Write;
use std::path::Path;

pub enum DocumentStorage {
    Zip(ZipPackage),
    Flat(FlatFile),
}

/// Document factory and operations.
pub struct Document {
    pub storage: DocumentStorage,
}

impl Document {
    pub fn open(path: &Path) -> Result<Self> {
        match detect_format(path)? {
            DetectedFormat::Zip => {
                let sp = ZipPackage::open_from_path(path)?;
                Ok(Self {
                    storage: DocumentStorage::Zip(sp),
                })
            }
            DetectedFormat::Flat => {
                let ff = FlatFile::open_from_path(path)?;
                Ok(Self {
                    storage: DocumentStorage::Flat(ff),
                })
            }
        }
    }

    pub fn new_flat() -> Self {
        Self {
            storage: DocumentStorage::Flat(FlatFile::new_empty()),
        }
    }

    pub fn new_package() -> Self {
        Self {
            storage: DocumentStorage::Zip(ZipPackage::new_empty()),
        }
    }

    pub fn read_content_xml(&mut self) -> Result<Vec<u8>> {
        match &mut self.storage {
            DocumentStorage::Zip(z) => z.read_entry("content.xml"),
            DocumentStorage::Flat(f) => f.read_entry("content.xml"),
        }
    }

    pub fn write_content_xml(&mut self, data: &[u8]) -> Result<()> {
        match &mut self.storage {
            DocumentStorage::Zip(z) => z.write_entry("content.xml", data),
            DocumentStorage::Flat(f) => f.write_entry("content.xml", data),
        }
    }

    pub fn save_as_package(self, path: &Path) -> Result<()> {
        // Build a minimal ODF package that LibreOffice can open:
        // - create a fresh zip
        // - write the `mimetype` entry first (stored, no compression)
        // - write `content.xml`
        // This avoids relying on archive ordering of an existing package.
        // Extract content.xml from whichever storage we have
        let content = match self.storage {
            DocumentStorage::Zip(mut z) => z.read_entry("content.xml")?,
            DocumentStorage::Flat(mut f) => f.read_entry("content.xml")?,
        };

        // produce a proper ODT package: mimetype (stored, first), META-INF/manifest.xml, content.xml, styles.xml, meta.xml
        use zip::write::FileOptions;
        use zip::CompressionMethod;

        let mut out = std::io::Cursor::new(Vec::new());
        let mut zipw = zip::ZipWriter::new(&mut out);

        let options_stored: FileOptions<'_, ()> =
            FileOptions::default().compression_method(CompressionMethod::Stored);
        let options_deflate: FileOptions<'_, ()> =
            FileOptions::default().compression_method(CompressionMethod::Deflated);

        // mimetype must be first entry and stored (no compression)
        let ext_owned = path
            .extension()
            .and_then(|s| s.to_str())
            .map(|s| s.to_lowercase());
        let mimetype = match ext_owned.as_deref() {
            Some("odt") | Some("fodt") => "application/vnd.oasis.opendocument.text",
            _ => "application/vnd.oasis.opendocument.text",
        };
        zipw.start_file("mimetype", options_stored)?;
        zipw.write_all(mimetype.as_bytes())?;

        // META-INF/manifest.xml (build with quick-xml)
        {
            use quick_xml::events::{BytesDecl, BytesEnd, BytesStart, Event as QEvent};
            use quick_xml::Writer as QWriter;
            let mut w = QWriter::new_with_indent(Vec::new(), b' ', 2);
            w.write_event(QEvent::Decl(BytesDecl::new("1.0", Some("UTF-8"), None)))?;
            let mut root = BytesStart::new("manifest:manifest");
            root.push_attribute((
                "xmlns:manifest",
                "urn:oasis:names:tc:opendocument:xmlns:manifest:1.0",
            ));
            w.write_event(QEvent::Start(root))?;

            let mut fe = BytesStart::new("manifest:file-entry");
            fe.push_attribute(("manifest:full-path", "/"));
            fe.push_attribute(("manifest:media-type", mimetype));
            w.write_event(QEvent::Empty(fe))?;

            let mut fe2 = BytesStart::new("manifest:file-entry");
            fe2.push_attribute(("manifest:full-path", "content.xml"));
            fe2.push_attribute((
                "manifest:media-type",
                "application/vnd.oasis.opendocument.text",
            ));
            w.write_event(QEvent::Empty(fe2))?;

            let mut fe3 = BytesStart::new("manifest:file-entry");
            fe3.push_attribute(("manifest:full-path", "styles.xml"));
            fe3.push_attribute((
                "manifest:media-type",
                "application/vnd.oasis.opendocument.styles",
            ));
            w.write_event(QEvent::Empty(fe3))?;

            let mut fe4 = BytesStart::new("manifest:file-entry");
            fe4.push_attribute(("manifest:full-path", "meta.xml"));
            fe4.push_attribute((
                "manifest:media-type",
                "application/vnd.oasis.opendocument.meta",
            ));
            w.write_event(QEvent::Empty(fe4))?;

            let mut fe5 = BytesStart::new("manifest:file-entry");
            fe5.push_attribute(("manifest:full-path", "settings.xml"));
            fe5.push_attribute((
                "manifest:media-type",
                "application/vnd.oasis.opendocument.settings",
            ));
            w.write_event(QEvent::Empty(fe5))?;

            w.write_event(QEvent::End(BytesEnd::new("manifest:manifest")))?; // end root
            zipw.start_file("META-INF/manifest.xml", options_deflate)?;
            zipw.write_all(&w.into_inner())?;
        }

        // content.xml
        zipw.start_file("content.xml", options_deflate)?;
        zipw.write_all(&content)?;

        // styles.xml using quick-xml
        {
            use quick_xml::events::{BytesDecl, BytesEnd, BytesStart, Event as QEvent};
            use quick_xml::Writer as QWriter;
            let mut w = QWriter::new_with_indent(Vec::new(), b' ', 2);
            w.write_event(QEvent::Decl(BytesDecl::new("1.0", Some("UTF-8"), None)))?;
            let mut root = BytesStart::new("office:document-styles");
            root.push_attribute((
                "xmlns:office",
                "urn:oasis:names:tc:opendocument:xmlns:office:1.0",
            ));
            root.push_attribute((
                "xmlns:style",
                "urn:oasis:names:tc:opendocument:xmlns:style:1.0",
            ));
            root.push_attribute((
                "xmlns:text",
                "urn:oasis:names:tc:opendocument:xmlns:text:1.0",
            ));
            root.push_attribute(("office:version", "1.4"));
            w.write_event(QEvent::Start(root))?;

            w.write_event(QEvent::Start(BytesStart::new("office:styles")))?;
            let mut s = BytesStart::new("style:style");
            s.push_attribute(("style:name", "Standard"));
            s.push_attribute(("style:family", "paragraph"));
            w.write_event(QEvent::Start(s))?;
            w.write_event(QEvent::Empty(BytesStart::new("style:properties")))?;
            w.write_event(QEvent::End(BytesEnd::new("style:style")))?;
            w.write_event(QEvent::End(BytesEnd::new("office:styles")))?;

            w.write_event(QEvent::Empty(BytesStart::new("office:automatic-styles")))?;
            w.write_event(QEvent::End(BytesEnd::new("office:document-styles")))?;
            zipw.start_file("styles.xml", options_deflate)?;
            zipw.write_all(&w.into_inner())?;
        }

        // meta.xml using quick-xml
        {
            use quick_xml::events::{BytesDecl, BytesEnd, BytesStart, BytesText, Event as QEvent};
            use quick_xml::Writer as QWriter;
            let mut w = QWriter::new_with_indent(Vec::new(), b' ', 2);
            w.write_event(QEvent::Decl(BytesDecl::new("1.0", Some("UTF-8"), None)))?;
            let mut root = BytesStart::new("office:document-meta");
            root.push_attribute((
                "xmlns:office",
                "urn:oasis:names:tc:opendocument:xmlns:office:1.0",
            ));
            root.push_attribute((
                "xmlns:meta",
                "urn:oasis:names:tc:opendocument:xmlns:meta:1.0",
            ));
            root.push_attribute(("xmlns:dc", "http://purl.org/dc/elements/1.1/"));
            root.push_attribute(("office:version", "1.4"));
            w.write_event(QEvent::Start(root))?;
            w.write_event(QEvent::Start(BytesStart::new("office:meta")))?;
            w.write_event(QEvent::Start(BytesStart::new("dc:title")))?;
            w.write_event(QEvent::Text(BytesText::new("Sample Document")))?;
            w.write_event(QEvent::End(BytesEnd::new("dc:title")))?;
            w.write_event(QEvent::Start(BytesStart::new("dc:creator")))?;
            w.write_event(QEvent::Text(BytesText::new("odferrous")))?;
            w.write_event(QEvent::End(BytesEnd::new("dc:creator")))?;
            w.write_event(QEvent::Start(BytesStart::new("meta:creation-date")))?;
            w.write_event(QEvent::Text(BytesText::new("2025-10-06T00:00:00")))?;
            w.write_event(QEvent::End(BytesEnd::new("meta:creation-date")))?;
            w.write_event(QEvent::Start(BytesStart::new("meta:generator")))?;
            w.write_event(QEvent::Text(BytesText::new("odferrous")))?;
            w.write_event(QEvent::End(BytesEnd::new("meta:generator")))?;
            w.write_event(QEvent::End(BytesEnd::new("office:meta")))?;
            w.write_event(QEvent::End(BytesEnd::new("office:document-meta")))?;
            zipw.start_file("meta.xml", options_deflate)?;
            zipw.write_all(&w.into_inner())?;
        }

        // settings.xml using quick-xml
        {
            use quick_xml::events::{BytesDecl, BytesEnd, BytesStart, Event as QEvent};
            use quick_xml::Writer as QWriter;
            let mut w = QWriter::new_with_indent(Vec::new(), b' ', 2);
            w.write_event(QEvent::Decl(BytesDecl::new("1.0", Some("UTF-8"), None)))?;
            let mut root = BytesStart::new("office:document-settings");
            root.push_attribute((
                "xmlns:office",
                "urn:oasis:names:tc:opendocument:xmlns:office:1.0",
            ));
            root.push_attribute(("office:version", "1.4"));
            w.write_event(QEvent::Start(root))?;
            w.write_event(QEvent::Empty(BytesStart::new("office:settings")))?;
            w.write_event(QEvent::End(BytesEnd::new("office:document-settings")))?;
            zipw.start_file("settings.xml", options_deflate)?;
            zipw.write_all(&w.into_inner())?;
        }

        zipw.finish()?;
        std::fs::write(path, out.into_inner())?;
        Ok(())
    }

    pub fn save_as_flat(self, path: &Path) -> Result<()> {
        // Build a simple, schema-friendly flat document wrapper from stored content.xml
        let content = match self.storage {
            DocumentStorage::Flat(mut f) => f.read_entry("content.xml")?,
            DocumentStorage::Zip(mut z) => z.read_entry("content.xml")?,
        };

        let s = String::from_utf8_lossy(&content);
        let body = if let Some(start) = s.find("<office:text") {
            if let Some(bstart) = s[start..].find('>') {
                let inner_start = start + bstart + 1;
                if let Some(end) = s.find("</office:text>") {
                    s[inner_start..end].to_string()
                } else {
                    String::new()
                }
            } else {
                String::new()
            }
        } else {
            // If content.xml is already a body fragment, use it as-is
            s.to_string()
        };

        // Build flat document with quick-xml writer
        {
            use quick_xml::events::{BytesDecl, BytesEnd, BytesStart, BytesText, Event as QEvent};
            use quick_xml::Writer as QWriter;
            let mut w = QWriter::new_with_indent(Vec::new(), b' ', 2);
            w.write_event(QEvent::Decl(BytesDecl::new("1.0", Some("UTF-8"), None)))?;
            let mut root = BytesStart::new("office:document");
            root.push_attribute((
                "xmlns:office",
                "urn:oasis:names:tc:opendocument:xmlns:office:1.0",
            ));
            root.push_attribute((
                "xmlns:meta",
                "urn:oasis:names:tc:opendocument:xmlns:meta:1.0",
            ));
            root.push_attribute(("xmlns:dc", "http://purl.org/dc/elements/1.1/"));
            root.push_attribute((
                "xmlns:text",
                "urn:oasis:names:tc:opendocument:xmlns:text:1.0",
            ));
            root.push_attribute(("office:version", "1.4"));
            root.push_attribute(("office:mimetype", "application/vnd.oasis.opendocument.text"));
            w.write_event(QEvent::Start(root))?;

            w.write_event(QEvent::Start(BytesStart::new("office:meta")))?;
            w.write_event(QEvent::Start(BytesStart::new("dc:title")))?;
            w.write_event(QEvent::Text(BytesText::new("Sample Document")))?;
            w.write_event(QEvent::End(BytesEnd::new("dc:title")))?;
            w.write_event(QEvent::Start(BytesStart::new("dc:creator")))?;
            w.write_event(QEvent::Text(BytesText::new("odferrous")))?;
            w.write_event(QEvent::End(BytesEnd::new("dc:creator")))?;
            w.write_event(QEvent::Start(BytesStart::new("meta:creation-date")))?;
            w.write_event(QEvent::Text(BytesText::new("2025-10-06T00:00:00")))?;
            w.write_event(QEvent::End(BytesEnd::new("meta:creation-date")))?;
            w.write_event(QEvent::End(BytesEnd::new("office:meta")))?;

            w.write_event(QEvent::Start(BytesStart::new("office:body")))?;
            w.write_event(QEvent::Start(BytesStart::new("office:text")))?;
            // Insert the body fragment by parsing it and copying events so XML elements are preserved
            {
                use quick_xml::Reader as QReader;
                let mut r = QReader::from_str(&body);
                let mut buf_r: Vec<u8> = Vec::new();
                loop {
                    match r.read_event_into(&mut buf_r)? {
                        QEvent::Start(e) => w.write_event(QEvent::Start(e.into_owned()))?,
                        QEvent::Empty(e) => w.write_event(QEvent::Empty(e.into_owned()))?,
                        QEvent::End(e) => w.write_event(QEvent::End(e.into_owned()))?,
                        QEvent::Text(e) => w.write_event(QEvent::Text(e.into_owned()))?,
                        QEvent::CData(e) => w.write_event(QEvent::CData(e.into_owned()))?,
                        QEvent::Comment(e) => w.write_event(QEvent::Comment(e.into_owned()))?,
                        QEvent::Eof => break,
                        _ => {}
                    }
                    buf_r.clear();
                }
            }
            w.write_event(QEvent::End(BytesEnd::new("office:text")))?;
            w.write_event(QEvent::End(BytesEnd::new("office:body")))?;

            w.write_event(QEvent::End(BytesEnd::new("office:document")))?;

            std::fs::write(path, w.into_inner())?;
        }

        Ok(())
    }
}
