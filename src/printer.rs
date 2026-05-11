use std::path::Path;
use std::process::Command;

use crate::error::{AppError, AppResult};

pub struct Printer {
    printer_name: String,
    print_command: String,
}

impl Printer {
    pub fn new() -> Self {
        Self {
            printer_name: "default".to_string(),
            print_command: "lp".to_string(),
        }
    }

    pub fn print(&self, path: &Path) -> AppResult<()> {
        if !path.exists() {
            return Err(AppError::FileNotFound(path.to_path_buf()));
        }

        log::info!("Sending to printer '{}': {:?}", self.printer_name, path);

        let mut cmd = Command::new(&self.print_command);
        
        #[cfg(target_os = "linux")]
        {
            cmd.arg("-d").arg(&self.printer_name);
        }
        
        #[cfg(target_os = "macos")]
        {
            cmd.arg("-P").arg(&self.printer_name);
        }
        
        #[cfg(target_os = "windows")]
        {
            cmd.arg("/d").arg(&self.printer_name);
        }

        cmd.arg(path);

        let output = cmd.output()
            .map_err(|e| AppError::Print(format!("Failed to execute print command: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(AppError::Print(format!(
                "Print command failed: {}",
                stderr.trim()
            )));
        }

        log::info!("Print job sent successfully");
        
        Ok(())
    }
}

impl Default for Printer {
    fn default() -> Self {
        Self::new()
    }
}