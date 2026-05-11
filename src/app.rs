use std::path::PathBuf;
use ratatui::{
    crossterm::{
        event::{self, DisableMouseCapture, EnableMouseCapture, KeyCode, KeyEventKind, KeyModifiers},
        execute,
        terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    },
    backend::CrosstermBackend,
    Terminal,
};

use crate::config::Config;
use crate::error::AppResult;
use crate::scanner::{Scanner, BarcodeFile};
use crate::preview::Preview;
use crate::printer::Printer;
use crate::tui::{AppState, AppMode, BarcodeFileUi, ConfigSnapshot};
use crate::ui::{render_ui, handle_key_event, TuiAction};

pub struct App {
    scanner: Scanner,
    preview: Preview,
    printer: Printer,
    config: Config,
    config_path: PathBuf,
}

impl App {
    pub fn new(config: Config, config_path: PathBuf) -> AppResult<Self> {
        let mut scanner = Scanner::new(config.barcode_directory.clone());
        scanner.scan()?;
        
        let preview = Preview::new(config.temp_directory.clone());
        
        Ok(Self {
            scanner,
            preview,
            printer: Printer::new(),
            config,
            config_path,
        })
    }

    pub fn run(&mut self) -> AppResult<()> {
        enable_raw_mode()?;
        let mut stdout = std::io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        let files = self.scanner.scan()?;
        let ui_files: Vec<BarcodeFileUi> = files
            .into_iter()
            .map(|f| BarcodeFileUi {
                name: f.name,
                path: f.path,
                size_kb: f.size / 1024,
                is_selected: false,
                quantity: 0,
            })
            .collect();

        let mut state = AppState::new(ui_files);
        state.current_config = Some(ConfigSnapshot {
            barcode_directory: self.config.barcode_directory.clone(),
            printer_name: self.config.printer_name.clone(),
            temp_directory: self.config.temp_directory.clone(),
            preview_command: self.config.preview_command.clone(),
            print_command: self.config.print_command.clone(),
        });
        let result = self.run_ui_loop(&mut terminal, &mut state);

        disable_raw_mode()?;
        execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )?;
        terminal.show_cursor()?;

        result
    }

    fn run_ui_loop<B: ratatui::backend::Backend>(
        &mut self,
        terminal: &mut Terminal<B>,
        state: &mut AppState,
    ) -> AppResult<()> {
        loop {
            terminal.draw(|f| render_ui(f, state))?;

            if let event::Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press
                    && key.modifiers.contains(KeyModifiers::CONTROL)
                    && key.code == KeyCode::Char('c')
                {
                    return Ok(());
                }

                if let Some(action) = handle_key_event(&key, state) {
                    match action {
                        TuiAction::Quit => return Ok(()),
                        TuiAction::StartPrint(paths) => {
                            self.execute_print(state, paths)?;
                        }
                        TuiAction::SaveConfig => {
                            if let Some(new_config) = state.get_config_from_state() {
                                if let Err(e) = new_config.save(&self.config_path) {
                                    eprintln!("Ошибка сохранения конфига: {}", e);
                                } else {
                                    self.config = new_config;
                                    self.scanner.directory = self.config.barcode_directory.clone();
                                    if let Err(e) = self.scanner.rescan() {
                                        eprintln!("Ошибка сканирования директории: {}", e);
                                    }
                                    let files = self.scanner.get_files();
                                    let ui_files: Vec<BarcodeFileUi> = files
                                        .into_iter()
                                        .map(|f| BarcodeFileUi {
                                            name: f.name,
                                            path: f.path,
                                            size_kb: f.size / 1024,
                                            is_selected: false,
                                            quantity: 0,
                                        })
                                        .collect();
                                    let mut new_state = AppState::new(ui_files);
                                    new_state.current_config = Some(ConfigSnapshot {
                                        barcode_directory: self.config.barcode_directory.clone(),
                                        printer_name: self.config.printer_name.clone(),
                                        temp_directory: self.config.temp_directory.clone(),
                                        preview_command: self.config.preview_command.clone(),
                                        print_command: self.config.print_command.clone(),
                                    });
                                    *state = new_state;
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    fn execute_print(&mut self, state: &mut AppState, items: Vec<(PathBuf, usize)>) -> AppResult<()> {
        state.mode = AppMode::Printing;

        let mut barcode_files: Vec<BarcodeFile> = Vec::new();
        
        for (path, qty) in items {
            let name = path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown")
                .to_string();
            
            for _ in 0..qty {
                barcode_files.push(BarcodeFile {
                    path: path.clone(),
                    name: name.clone(),
                    size: 0,
                });
            }
        }

        let result = self.preview.generate_preview(&barcode_files);
        
        match result {
            Ok(pdf_path) => {
                if self.printer.print(&pdf_path).is_ok() {
                    state.mode = AppMode::Success;
                } else {
                    state.mode = AppMode::Error;
                }
            }
            Err(_) => {
                state.mode = AppMode::Error;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_app_creation() {
        let tmp_dir = TempDir::new().unwrap();
        let config = Config {
            barcode_directory: tmp_dir.path().to_path_buf(),
            ..Config::default()
        };
        
        let app = App::new(config, PathBuf::from("test.json")).unwrap();
        assert_eq!(app.scanner.directory, tmp_dir.path().to_path_buf());
    }
}