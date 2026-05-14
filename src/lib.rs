pub mod document;
pub mod dom;
pub mod error;
pub mod storage;
pub mod text_document;

pub use crate::error::{OdfError, Result};
pub use crate::text_document::TextDocument;

/// Prelude for convenient imports
pub mod prelude {
    pub use crate::error::{OdfError, Result};
    pub use crate::text_document::TextDocument;
}
