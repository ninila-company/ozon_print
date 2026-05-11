use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::fs;

use crate::error::{AppError, AppResult};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub barcode_directory: PathBuf,
    pub printer_name: String,
    pub page_size: PageSize,
    pub temp_directory: PathBuf,
    pub preview_command: String,
    pub print_command: String,
    pub max_recent_files: usize,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
#[serde(rename_all = "UPPERCASE")]
pub enum PageSize {
    A4,
    A5,
    Letter,
    #[default]
    Custom,
}

impl Default for Config {
    fn default() -> Self {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        Self {
            barcode_directory: home.join("barcodes"),
            printer_name: "default".to_string(),
            page_size: PageSize::A4,
            temp_directory: std::env::temp_dir().join("ozon_print"),
            preview_command: "xdg-open".to_string(),
            print_command: "lp".to_string(),
            max_recent_files: 10,
        }
    }
}

impl Config {
    pub fn load(path: &PathBuf) -> AppResult<Self> {
        if !path.exists() {
            return Err(AppError::FileNotFound(path.clone()));
        }

        let content = fs::read_to_string(path)
            .map_err(|e| AppError::Config(format!("Failed to read config: {}", e)))?;

        let config: Config = serde_json::from_str(&content)
            .map_err(|e| AppError::Config(format!("Failed to parse config: {}", e)))?;

        config.validate()?;
        Ok(config)
    }

    pub fn validate(&self) -> AppResult<()> {
        if !self.barcode_directory.exists() {
            return Err(AppError::DirectoryNotFound(self.barcode_directory.clone()));
        }

        if !self.temp_directory.exists() {
            fs::create_dir_all(&self.temp_directory)?;
        }

        Ok(())
    }

    pub fn merge_with_args(
        mut self,
        directory: Option<PathBuf>,
        printer: Option<String>,
    ) -> Self {
        if let Some(dir) = directory {
            self.barcode_directory = dir;
        }
        if let Some(printer_name) = printer {
            self.printer_name = printer_name;
        }
        self
    }

    pub fn save(&self, path: &PathBuf) -> AppResult<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| AppError::Config(format!("Failed to create config directory: {}", e)))?;
        }
        
        let content = serde_json::to_string_pretty(self)
            .map_err(|e| AppError::Config(format!("Failed to serialize config: {}", e)))?;
        
        fs::write(path, content)
            .map_err(|e| AppError::Config(format!("Failed to write config: {}", e)))?;
        
        Ok(())
    }
}