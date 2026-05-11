use std::path::{Path, PathBuf};
use std::fs;

use lopdf::{Document, Object, Dictionary, Stream};
use crate::error::{AppError, AppResult};
use crate::scanner::BarcodeFile;

const PAGE_WIDTH_MM: f32 = 80.0;
const PAGE_HEIGHT_MM: f32 = 40.0;

pub struct Preview {
    temp_directory: PathBuf,
    temp_file: Option<PathBuf>,
}

impl Preview {
    pub fn new(temp_directory: PathBuf) -> Self {
        Self {
            temp_directory,
            temp_file: None,
        }
    }

    pub fn generate_preview(&mut self, files: &[BarcodeFile]) -> AppResult<PathBuf> {
        if files.is_empty() {
            return Err(AppError::Pdf("No files selected".to_string()));
        }

        if !self.temp_directory.exists() {
            fs::create_dir_all(&self.temp_directory)?;
        }

        let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
        let output_path = self.temp_directory.join(format!("temp_print_{}.pdf", timestamp));
        
        self.combine_pdfs(files, &output_path)?;
        self.temp_file = Some(output_path.clone());
        
        Ok(output_path)
    }

    fn combine_pdfs(&self, files: &[BarcodeFile], output: &Path) -> AppResult<()> {
        let pt_per_mm = 2.834645669;
        let page_width_pt = PAGE_WIDTH_MM * pt_per_mm;
        let page_height_pt = PAGE_HEIGHT_MM * pt_per_mm;

        let mut target_doc = Document::with_version("1.5");

        let mut all_pages: Vec<Object> = Vec::new();

        for barcode_file in files {
            let source_doc = Document::load(&barcode_file.path)
                .map_err(|e| AppError::Pdf(format!("Failed to load PDF {:?}: {}", barcode_file.path, e)))?;

            let source_pages = source_doc.get_pages();
            
            for (_page_num, page_id) in source_pages.iter() {
                let content_stream = self.extract_page_content(&source_doc, *page_id)?;
                let resources = self.get_page_resources(&source_doc, *page_id)?;

                let content_obj = target_doc.add_object(Stream::new(
                    Dictionary::new(),
                    content_stream,
                ));

                let mut page_dict = Dictionary::new();
                page_dict.set("Type", "Page");
                page_dict.set("MediaBox", Object::Array(vec![
                    Object::Real(0.0),
                    Object::Real(0.0),
                    Object::Real(page_width_pt),
                    Object::Real(page_height_pt),
                ]));
                page_dict.set("Contents", content_obj);
                page_dict.set("Resources", resources);

                let page_ref = target_doc.add_object(Object::Dictionary(page_dict));
                all_pages.push(Object::Reference(page_ref));
            }
        }

        let pages_dict = Dictionary::from_iter([
            (b"Type".to_vec(), Object::Name(b"Pages".to_vec())),
            (b"Count".to_vec(), Object::Integer(all_pages.len() as i64)),
            (b"Kids".to_vec(), Object::Array(all_pages.clone())),
        ]);
        
        let pages_ref = target_doc.add_object(Object::Dictionary(pages_dict));

        for page_obj in &all_pages {
            if let Object::Reference(page_ref) = page_obj {
                if let Ok(page_dict) = target_doc.get_dictionary_mut(*page_ref) {
                    let _ = page_dict.set("Parent", Object::Reference(pages_ref));
                }
            }
        }

        let catalog = target_doc.catalog_mut()
            .map_err(|e| AppError::Pdf(format!("Failed to get catalog: {}", e)))?;
        let _ = catalog.set("Pages", Object::Reference(pages_ref));

        target_doc.trailer.set("Size", Object::Integer(target_doc.objects.len() as i64 + 1));

        target_doc.save(output)
            .map_err(|e| AppError::Pdf(format!("Failed to save combined PDF: {}", e)))?;

        Ok(())
    }

    fn extract_page_content(&self, doc: &Document, page_id: lopdf::ObjectId) -> AppResult<Vec<u8>> {
        let page_dict = doc.get_dictionary(page_id)
            .map_err(|_| AppError::Pdf("Failed to get page dictionary".to_string()))?;

        let contents_ref = page_dict.get(b"Contents")
            .map_err(|_| AppError::Pdf("No Contents in page".to_string()))?;
        
        let mut content_data = Vec::new();
        
        match contents_ref {
            Object::Reference(r) => {
                if let Ok(obj) = doc.get_object(*r) {
                    content_data = self.extract_stream_data(&obj);
                }
            }
            Object::Array(arr) => {
                for item in arr {
                    if let Object::Reference(r) = item {
                        if let Ok(obj) = doc.get_object(*r) {
                            let mut data = self.extract_stream_data(&obj);
                            content_data.append(&mut data);
                            content_data.push(b'\n');
                        }
                    }
                }
            }
            Object::Stream(stream) => {
                content_data = stream.content.clone();
            }
            _ => {}
        }

        Ok(content_data)
    }

    fn get_page_resources(&self, doc: &Document, page_id: lopdf::ObjectId) -> AppResult<Object> {
        let page_dict = doc.get_dictionary(page_id)
            .map_err(|_| AppError::Pdf("Failed to get page dictionary".to_string()))?;

        let resources = page_dict.get(b"Resources")
            .cloned()
            .unwrap_or(Object::Dictionary(Dictionary::new()));

        Ok(resources)
    }

    fn extract_stream_data(&self, obj: &Object) -> Vec<u8> {
        match obj {
            Object::Stream(stream) => stream.content.clone(),
            _ => Vec::new(),
        }
    }

    pub fn cleanup(&mut self) {
        if let Some(path) = self.temp_file.take() {
            if path.exists() {
                if let Err(e) = fs::remove_file(&path) {
                    log::warn!("Failed to cleanup temp file {:?}: {}", path, e);
                }
            }
        }
    }
}

impl Drop for Preview {
    fn drop(&mut self) {
        self.cleanup();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_preview_creation() {
        let tmp_dir = TempDir::new().unwrap();
        let preview = Preview::new(tmp_dir.path().to_path_buf());
        assert!(preview.temp_file.is_none());
    }
}