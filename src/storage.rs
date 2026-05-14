use crate::error::{OdfError, Result};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::Path;

/// Storage abstraction for ODF containers.
pub trait Storage {
    /// Read the content of a named entry (e.g., "content.xml") into a Vec<u8>.
    fn read_entry(&mut self, name: &str) -> Result<Vec<u8>>;

    /// Write/replace the named entry.
    fn write_entry(&mut self, name: &str, data: &[u8]) -> Result<()>;

    /// Persist the storage to path.
    fn persist(self: Box<Self>, path: &Path) -> Result<()>;
}

/// Zip based package storage (ODT/ODS)
pub struct ZipPackage {
    // we'll hold the full file in memory for simplicity in this bootstrap
    cursor: std::io::Cursor<Vec<u8>>,
}

impl ZipPackage {
    pub fn open_from_path(path: &Path) -> Result<Self> {
        let data = std::fs::read(path)?;
        Ok(Self {
            cursor: std::io::Cursor::new(data),
        })
    }

    pub fn new_empty() -> Self {
        // Start with an empty zip container
        Self {
            cursor: std::io::Cursor::new(Vec::new()),
        }
    }
}

impl Storage for ZipPackage {
    fn read_entry(&mut self, name: &str) -> Result<Vec<u8>> {
        self.cursor.seek(SeekFrom::Start(0))?;
        let mut archive = zip::ZipArchive::new(&mut self.cursor)?;
        let mut file = archive.by_name(name)?;
        let mut buf = Vec::new();
        file.read_to_end(&mut buf)?;
        Ok(buf)
    }

    fn write_entry(&mut self, name: &str, data: &[u8]) -> Result<()> {
        // Rebuild archive in-memory replacing/adding the entry.
        self.cursor.seek(SeekFrom::Start(0))?;
        let archive =
            zip::ZipArchive::new(&mut self.cursor).map_err(|_| zip::result::ZipError::FileNotFound);

        // Read existing entries
        let mut entries: Vec<(String, Vec<u8>)> = Vec::new();
        if let Ok(mut a) = archive {
            for i in 0..a.len() {
                let mut f = a.by_index(i)?;
                let mut b = Vec::new();
                f.read_to_end(&mut b)?;
                entries.push((f.name().to_string(), b));
            }
        }

        // Replace or insert
        let mut replaced = false;
        for (n, b) in entries.iter_mut() {
            if n == name {
                *b = data.to_vec();
                replaced = true;
            }
        }
        if !replaced {
            entries.push((name.to_string(), data.to_vec()));
        }

        // Write new archive
        let mut out = std::io::Cursor::new(Vec::new());
        let mut w = zip::ZipWriter::new(&mut out);
        let options: zip::write::FileOptions<'_, ()> =
            zip::write::FileOptions::default().compression_method(zip::CompressionMethod::Stored);
        for (n, b) in entries {
            w.start_file(n, options)?;
            w.write_all(&b)?;
        }
        w.finish()?;
        self.cursor = out;
        Ok(())
    }

    fn persist(self: Box<Self>, path: &Path) -> Result<()> {
        std::fs::write(path, self.cursor.into_inner())?;
        Ok(())
    }
}

/// Flat XML storage (FODT/FODS)
pub struct FlatFile {
    data: Vec<u8>,
}

impl FlatFile {
    pub fn open_from_path(path: &Path) -> Result<Self> {
        let data = std::fs::read(path)?;
        Ok(Self { data })
    }

    pub fn new_empty() -> Self {
        Self { data: Vec::new() }
    }
}

impl Storage for FlatFile {
    fn read_entry(&mut self, _name: &str) -> Result<Vec<u8>> {
        // Flat file has the XML as the whole payload
        Ok(self.data.clone())
    }

    fn write_entry(&mut self, _name: &str, data: &[u8]) -> Result<()> {
        self.data = data.to_vec();
        Ok(())
    }

    fn persist(self: Box<Self>, path: &Path) -> Result<()> {
        std::fs::write(path, self.data)?;
        Ok(())
    }
}

/// Detect format by file extension or magic bytes.
pub enum DetectedFormat {
    Zip,
    Flat,
}

pub fn detect_format(path: &Path) -> Result<DetectedFormat> {
    if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
        match ext.to_lowercase().as_str() {
            "odt" | "ods" | "ott" => return Ok(DetectedFormat::Zip),
            "fodt" | "fods" | "xml" => return Ok(DetectedFormat::Flat),
            _ => {}
        }
    }

    // Inspect magic bytes
    let mut f = std::fs::File::open(path)?;
    let mut buf = [0u8; 16];
    let n = f.read(&mut buf)?;
    let s = std::str::from_utf8(&buf[..n]).unwrap_or("");
    if s.contains("<office:document") || s.contains("<?xml") {
        return Ok(DetectedFormat::Flat);
    }

    // PK zip magic
    if &buf[..2] == b"PK" {
        return Ok(DetectedFormat::Zip);
    }

    Err(OdfError::InvalidFormat("Unknown file format".into()))
}
