use crate::document::Document;
use crate::dom::DocumentModel;
use crate::error::Result;

/// A minimal text document with paragraphs and headings.
///
/// # Examples
///
/// Zip package example:
///
/// ```no_run
/// use odferrous::TextDocument;
/// let mut doc = TextDocument::new_package();
/// doc.add_paragraph("Hello package");
/// let p = std::path::Path::new("out.odt");
/// doc.save_as_package(p).unwrap();
/// ```
///
/// Flat file example:
///
/// ```no_run
/// use odferrous::TextDocument;
/// let mut doc = TextDocument::new_flat();
/// doc.add_heading("Title");
/// let p = std::path::Path::new("out.fodt");
/// doc.save_as_flat(p).unwrap();
/// ```
pub struct TextDocument {
    pub doc: Document,
    paragraphs: Vec<String>,
    headings: Vec<String>,
}

impl TextDocument {
    pub fn open<P: AsRef<std::path::Path>>(path: P) -> Result<Self> {
        let mut d = Document::open(path.as_ref())?;
        // Use DocumentModel to parse content.xml
        let mut paragraphs = Vec::new();
        let mut headings = Vec::new();
        if let Ok(data) = d.read_content_xml() {
            if let Ok(model) = DocumentModel::from_content_xml(&data) {
                paragraphs = model.paragraphs;
                headings = model.headings;
            }
        }
        Ok(Self {
            doc: d,
            paragraphs,
            headings,
        })
    }

    pub fn new_flat() -> Self {
        Self {
            doc: Document::new_flat(),
            paragraphs: Vec::new(),
            headings: Vec::new(),
        }
    }

    pub fn new_package() -> Self {
        Self {
            doc: Document::new_package(),
            paragraphs: Vec::new(),
            headings: Vec::new(),
        }
    }

    pub fn add_paragraph(&mut self, text: &str) {
        self.paragraphs.push(text.to_string());
    }

    pub fn add_heading(&mut self, text: &str) {
        self.headings.push(text.to_string());
    }

    fn build_content_xml(&self) -> Result<Vec<u8>> {
        let model = DocumentModel {
            headings: self.headings.clone(),
            paragraphs: self.paragraphs.clone(),
        };
        Ok(model.to_content_xml())
    }

    pub fn save_as_package<P: AsRef<std::path::Path>>(&mut self, path: P) -> Result<()> {
        let data = self.build_content_xml()?;
        self.doc.write_content_xml(&data)?;
        let doc = std::mem::replace(&mut self.doc, Document::new_package());
        doc.save_as_package(path.as_ref())
    }

    pub fn save_as_flat<P: AsRef<std::path::Path>>(&mut self, path: P) -> Result<()> {
        let data = self.build_content_xml()?;
        self.doc.write_content_xml(&data)?;
        let doc = std::mem::replace(&mut self.doc, Document::new_flat());
        doc.save_as_flat(path.as_ref())
    }
}
