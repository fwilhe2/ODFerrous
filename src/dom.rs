use crate::error::{OdfError, Result};

/// A minimal in-memory DOM representing headings and paragraphs for ODF text documents.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocumentModel {
    pub headings: Vec<String>,
    pub paragraphs: Vec<String>,
}

impl Default for DocumentModel {
    fn default() -> Self {
        Self::new()
    }
}

impl DocumentModel {
    pub fn new() -> Self {
        Self {
            headings: Vec::new(),
            paragraphs: Vec::new(),
        }
    }

    pub fn to_content_xml(&self) -> Vec<u8> {
        use quick_xml::events::{BytesDecl, BytesEnd, BytesStart, BytesText, Event};
        use quick_xml::Writer;

        let mut writer = Writer::new_with_indent(Vec::new(), b' ', 2);
        // XML declaration
        writer
            .write_event(Event::Decl(BytesDecl::new("1.0", Some("UTF-8"), None)))
            .ok();

        // office:document-content start with namespace attributes
        let mut root = BytesStart::new("office:document-content");
        root.push_attribute((
            "xmlns:office",
            "urn:oasis:names:tc:opendocument:xmlns:office:1.0",
        ));
        root.push_attribute((
            "xmlns:text",
            "urn:oasis:names:tc:opendocument:xmlns:text:1.0",
        ));
        root.push_attribute(("office:version", "1.4"));
        writer.write_event(Event::Start(root)).ok();

        // <office:body>
        writer
            .write_event(Event::Start(BytesStart::new("office:body")))
            .ok();
        // <office:text>
        writer
            .write_event(Event::Start(BytesStart::new("office:text")))
            .ok();

        for h in &self.headings {
            let mut el = BytesStart::new("text:h");
            el.push_attribute(("text:outline-level", "1"));
            writer.write_event(Event::Start(el)).ok();
            writer.write_event(Event::Text(BytesText::new(h))).ok();
            writer.write_event(Event::End(BytesEnd::new("text:h"))).ok();
        }

        for p in &self.paragraphs {
            writer
                .write_event(Event::Start(BytesStart::new("text:p")))
                .ok();
            writer.write_event(Event::Text(BytesText::new(p))).ok();
            writer.write_event(Event::End(BytesEnd::new("text:p"))).ok();
        }

        // close office:text, office:body, office:document-content
        writer
            .write_event(Event::End(BytesEnd::new("office:text")))
            .ok();
        writer
            .write_event(Event::End(BytesEnd::new("office:body")))
            .ok();
        writer
            .write_event(Event::End(BytesEnd::new("office:document-content")))
            .ok();

        writer.into_inner()
    }

    pub fn from_content_xml(bytes: &[u8]) -> Result<Self> {
        use quick_xml::events::Event;
        use quick_xml::Reader;
        // quick-xml works on bytes, but we require valid UTF-8 input for ODF XML
        if std::str::from_utf8(bytes).is_err() {
            return Err(OdfError::Other("Invalid UTF-8 in XML input".into()));
        }

        let mut reader = Reader::from_reader(bytes);
        let mut buf = Vec::new();
        let mut model = DocumentModel::new();
        // track whether we've seen a root start and its closing to detect malformed/incomplete XML
        let mut saw_root_start = false;
        let mut saw_root_end = false;
        // no need for a current state; we extract text directly on Start events

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) => {
                    // Namespace-aware: extract local name from QName (handles prefixes and {ns}local forms)
                    // convert qname to an owned string to avoid temporary borrow issues
                    let qname_str = String::from_utf8_lossy(e.name().as_ref()).into_owned();
                    let local_owned = if let Some(colon) = qname_str.find(':') {
                        qname_str[colon + 1..].to_string()
                    } else if qname_str.starts_with('{') {
                        // expanded form: {namespace}local
                        if let Some(close) = qname_str.find('}') {
                            qname_str[close + 1..].to_string()
                        } else {
                            qname_str.clone()
                        }
                    } else {
                        qname_str.clone()
                    };

                    if local_owned == "document-content" {
                        saw_root_start = true;
                    }

                    match local_owned.as_str() {
                        "h" => {
                            if let Ok(txt) = reader.read_text(e.name()) {
                                let s = String::from_utf8_lossy(txt.as_ref()).into_owned();
                                model.headings.push(unescape_xml(&s));
                            }
                        }
                        "p" => {
                            if let Ok(txt) = reader.read_text(e.name()) {
                                let s = String::from_utf8_lossy(txt.as_ref()).into_owned();
                                model.paragraphs.push(unescape_xml(&s));
                            }
                        }
                        _ => {}
                    }
                }
                Ok(Event::End(ref e)) => {
                    let qname_str = String::from_utf8_lossy(e.name().as_ref()).into_owned();
                    let local_owned = if let Some(colon) = qname_str.find(':') {
                        qname_str[colon + 1..].to_string()
                    } else if qname_str.starts_with('{') {
                        if let Some(close) = qname_str.find('}') {
                            qname_str[close + 1..].to_string()
                        } else {
                            qname_str.clone()
                        }
                    } else {
                        qname_str.clone()
                    };

                    if local_owned == "document-content" {
                        saw_root_end = true;
                    }
                }
                Ok(Event::Eof) => break,
                Err(e) => return Err(OdfError::Other(format!("XML parse error: {}", e))),
                _ => {}
            }
            buf.clear();
        }

        // If root was started but not properly closed, treat as malformed
        if saw_root_start && !saw_root_end {
            return Err(OdfError::Other(
                "Unexpected EOF: unclosed document-content".into(),
            ));
        }

        Ok(model)
    }
}

fn unescape_xml(s: &str) -> String {
    s.replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&amp;", "&")
        .replace("&quot;", "\"")
        .replace("&apos;", "'")
}
