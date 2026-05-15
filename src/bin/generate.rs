use odferrous::TextDocument;
use std::path::Path;


use std::io::{Read, Seek, SeekFrom, Write};
use std::sync::Arc;

// --- Errors ---
#[derive(Debug, thiserror::Error)]
pub enum OdfError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Failed to parse XML: {0}")]
    XmlParse(String),
    #[error("Unknown format")]
    UnknownFormat,
}

pub type Result<T> = std::result::Result<T, OdfError>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum OdfFormat {
    /// Standard zipped package containing content.xml, meta.xml, styles.xml, etc.
    #[default]
    Zipped,
    /// A single, flat XML file containing the entire document tree.
    FlatXml,
}


// --- Core Types ---
#[derive(Clone, Debug)]
pub struct OdfDocument {
    // Structural sharing using Arc allows cheap cloning of the tree
    root: Arc<DocumentRoot>,
}

#[derive(Clone, Debug)]
struct DocumentRoot {
    meta: DocumentMeta,
    body: Vec<Arc<OdfElement>>,
}

#[derive(Clone, Debug)]
pub struct DocumentMeta {
    pub title: Option<String>,
    pub creator: Option<String>,
}

#[derive(Clone, Debug)]
pub enum OdfElement {
    Paragraph(String),
    Heading { level: u32, text: String },
}

// --- API Implementation ---
impl OdfDocument {

    /// Parse a document, auto-detecting whether it is Zipped or Flat XML.
    ///
    /// Requires `Seek` so we can peek at the headers and reset the cursor.
    pub fn parse<R: Read + Seek>(mut reader: R) -> Result<Self> {
        let mut magic_bytes = [0u8; 4];
        reader.read_exact(&mut magic_bytes)?;

        // Rewind the reader back to the beginning after sniffing
        reader.seek(SeekFrom::Start(0))?;

        // "PK\x03\x04" is the standard zip magic number
        if &magic_bytes[0..2] == b"PK" {
            Self::parse_zipped(reader)
        } else if magic_bytes[0] == b'<' {
            Self::parse_flat_xml(reader)
        } else {
            Err(OdfError::UnknownFormat)
        }
    }

    /// Explicitly write the document in the desired format.
    pub fn write<W: Write>(&self, writer: W, format: OdfFormat) -> Result<()> {
        match format {
            OdfFormat::Zipped => self.serialize_to_zip(writer),
            OdfFormat::FlatXml => self.serialize_to_flat_xml(writer),
        }
    }


    // --- Private Serialization Helpers ---
    fn serialize_to_flat_xml<W: Write>(&self, mut _writer: W) -> Result<()> {
        // Here you would dump everything into a single <office:document> root element
        //todo!()
        println!("foo");

        Ok(())
    }

    fn serialize_to_zip<W: Write>(&self, mut _writer: W) -> Result<()> {
        // Here you would use a crate like `zip` to create the container,
        // writing "mimetype", "content.xml", "styles.xml", etc.
        // todo!()

        println!("foo");

        Ok(())
    }

    fn parse_zipped<R: Read + Seek>(mut _reader: R) -> Result<Self> { todo!() }
    fn parse_flat_xml<R: Read>(_reader: R) -> Result<Self> { todo!() }


    /// Start building a brand new document
    pub fn builder() -> OdfDocumentBuilder {
        OdfDocumentBuilder::default()
    }

    /// Immutable modification: Modifies the document by creating a new one,
    /// leveraging Arc for fast structural sharing.
    pub fn with_title(&self, title: impl Into<String>) -> Self {
        let mut new_root = (*self.root).clone();
        new_root.meta.title = Some(title.into());

        OdfDocument {
            root: Arc::new(new_root),
        }
    }

    pub fn with_paragraph(&self, text: impl Into<String>) -> Self {
        let mut new_root = (*self.root).clone();
        new_root.body.push(Arc::new(OdfElement::Paragraph(text.into())));

        OdfDocument {
            root: Arc::new(new_root),
        }
    }
}

// --- Builder Pattern ---
#[derive(Default)]
pub struct OdfDocumentBuilder {
    title: Option<String>,
    creator: Option<String>,
    elements: Vec<OdfElement>,
}

impl OdfDocumentBuilder {
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    pub fn creator(mut self, creator: impl Into<String>) -> Self {
        self.creator = Some(creator.into());
        self
    }

    pub fn add_heading(mut self, level: u32, text: impl Into<String>) -> Self {
        self.elements.push(OdfElement::Heading { level, text: text.into() });
        self
    }

    pub fn build(self) -> Result<OdfDocument> {
        let body = self.elements.into_iter().map(Arc::new).collect();
        Ok(OdfDocument {
            root: Arc::new(DocumentRoot {
                meta: DocumentMeta { title: self.title, creator: self.creator },
                body,
            }),
        })
    }
}

fn main() -> Result<()> {
    // 1. Build a document immutably
    let doc = OdfDocument::builder()
        .title("Version Controlled Doc")
        .add_heading(1, "Git-friendly ODF")
        .build()?;

    // 2. Write as Flat XML for clean git diffs
    let mut flat_file = std::fs::File::create("document.fodt")?;
    doc.write(&mut flat_file, OdfFormat::FlatXml)?;

    // 3. Write as standard zipped ODF for LibreOffice/MS Word distribution
    let mut zipped_file = std::fs::File::create("document.odt")?;
    doc.write(&mut zipped_file, OdfFormat::Zipped)?;

    // 4. Create a new document using immutable api
    let new_doc = doc.with_paragraph("hello world");

    // 5. Reading is seamless; the library sniffs the file type automatically
    let opened_file = std::fs::File::open("document.fodt")?;
    let _loaded_doc = OdfDocument::parse(std::io::BufReader::new(opened_file))?;

    Ok(())
}