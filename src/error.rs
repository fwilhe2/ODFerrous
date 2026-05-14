use thiserror::Error;

/// Common error type for ODFerrous.
#[derive(Error, Debug)]
pub enum OdfError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Zip error: {0}")]
    Zip(#[from] zip::result::ZipError),

    #[error("XML parse error: {0}")]
    Xml(#[from] quick_xml::Error),

    #[error("XML serialize error: {0}")]
    XmlSerialize(#[from] quick_xml::se::SeError),

    #[error("Invalid format: {0}")]
    InvalidFormat(String),

    #[error("Other error: {0}")]
    Other(String),
}

pub type Result<T> = std::result::Result<T, OdfError>;
