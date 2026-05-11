use std::path::{Path, PathBuf};
use std::fs;

use crate::error::{AppError, AppResult};

#[derive(Debug, Clone)]
pub struct BarcodeFile {
    pub path: PathBuf,
    pub name: String,
    pub size: u64,
}

impl BarcodeFile {
    pub fn from_path(path: &Path) -> AppResult<Self> {
        let metadata = fs::metadata(path)?;
        
        Ok(Self {
            path: path.to_path_buf(),
            name: path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown")
                .to_string(),
            size: metadata.len(),
        })
    }

    pub fn is_pdf(path: &Path) -> bool {
        path.extension()
            .and_then(|e| e.to_str())
            .map(|e| e.to_lowercase() == "pdf")
            .unwrap_or(false)
    }
}

pub struct Scanner {
    pub directory: PathBuf,
    files: Vec<BarcodeFile>,
}

impl Scanner {
    pub fn new(directory: PathBuf) -> Self {
        let files = Vec::new();
        Self { directory, files }
    }

    pub fn scan(&mut self) -> AppResult<Vec<BarcodeFile>> {
        if !self.directory.exists() {
            return Err(AppError::DirectoryNotFound(self.directory.clone()));
        }

        let mut files = Vec::new();
        
        for entry in fs::read_dir(&self.directory)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_file() && BarcodeFile::is_pdf(&path) {
                match BarcodeFile::from_path(&path) {
                    Ok(barcode) => files.push(barcode),
                    Err(e) => log::warn!("Skipping invalid file {:?}: {}", path, e),
                }
            }
        }

        files.sort_by(|a, b| a.name.cmp(&b.name));
        self.files = files.clone();
        Ok(files)
    }

    pub fn rescan(&mut self) -> AppResult<()> {
        self.scan()?;
        Ok(())
    }

    pub fn get_files(&self) -> Vec<BarcodeFile> {
        self.files.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::TempDir;

    #[test]
    fn test_scanner_creates_barcode_file() {
        let tmp_dir = TempDir::new().unwrap();
        let test_file_path = tmp_dir.path().join("test.pdf");
        
        let mut file = fs::File::create(&test_file_path).unwrap();
        file.write_all(b"%PDF-1.4 test content").unwrap();
        drop(file);

        let barcode = BarcodeFile::from_path(&test_file_path).unwrap();
        assert_eq!(barcode.name, "test.pdf");
        assert!(barcode.size > 0);
    }

    #[test]
    fn test_is_pdf() {
        assert!(BarcodeFile::is_pdf(Path::new("test.pdf")));
        assert!(BarcodeFile::is_pdf(Path::new("test.PDF")));
        assert!(!BarcodeFile::is_pdf(Path::new("test.txt")));
        assert!(!BarcodeFile::is_pdf(Path::new("test")));
    }
}