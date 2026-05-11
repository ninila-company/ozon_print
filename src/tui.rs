use std::path::PathBuf;

#[derive(Debug, Clone, Default)]
pub struct AppState {
    pub files: Vec<BarcodeFileUi>,
    pub filtered_files: Vec<usize>,
    pub selected: Vec<usize>,
    pub cursor: usize,
    pub scroll_offset: usize,
    pub mode: AppMode,
    pub input_buffer: String,
    pub editing_index: Option<usize>,
    pub search_query: String,
    pub is_searching: bool,
    pub config_edit_state: Option<ConfigEditState>,
    pub current_config: Option<ConfigSnapshot>,
}

#[derive(Debug, Clone)]
pub struct ConfigSnapshot {
    pub barcode_directory: PathBuf,
    pub printer_name: String,
    pub temp_directory: PathBuf,
    pub preview_command: String,
    pub print_command: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AppMode {
    Select,
    ConfirmPrint,
    Printing,
    Success,
    Error,
    ConfigEdit,
}

impl Default for AppMode {
    fn default() -> Self {
        AppMode::Select
    }
}

#[derive(Debug, Clone)]
pub struct ConfigEditState {
    pub barcode_directory: String,
    pub printer_name: String,
    pub temp_directory: String,
    pub preview_command: String,
    pub print_command: String,
    pub selected_field: usize,
    pub is_editing: bool,
    pub input_buffer: String,
}

#[derive(Debug, Clone)]
pub struct BarcodeFileUi {
    pub name: String,
    pub path: PathBuf,
    pub size_kb: u64,
    pub is_selected: bool,
    pub quantity: usize,
}

impl AppState {
    pub fn new(files: Vec<BarcodeFileUi>) -> Self {
        let filtered: Vec<usize> = (0..files.len()).collect();
        Self {
            files,
            filtered_files: filtered,
            input_buffer: String::new(),
            editing_index: None,
            search_query: String::new(),
            is_searching: false,
            ..Default::default()
        }
    }

    pub fn start_search(&mut self) {
        self.is_searching = true;
        self.input_buffer.clear();
        self.search_query.clear();
        self.filtered_files = (0..self.files.len()).collect();
        self.cursor = 0;
        self.scroll_offset = 0;
    }

    pub fn stop_search(&mut self) {
        self.is_searching = false;
        self.input_buffer.clear();
    }

    pub fn update_search(&mut self) {
        self.search_query = self.input_buffer.clone();
        
        if self.search_query.is_empty() {
            self.filtered_files = (0..self.files.len()).collect();
        } else {
            let query_lower = self.search_query.to_lowercase();
            self.filtered_files = self.files
                .iter()
                .enumerate()
                .filter(|(_, f)| f.name.to_lowercase().contains(&query_lower))
                .map(|(i, _)| i)
                .collect();
        }
        
        self.cursor = 0;
        self.scroll_offset = 0;
        self.selected.retain(|&idx| self.filtered_files.contains(&idx));
    }

    pub fn display_count(&self) -> usize {
        self.filtered_files.len()
    }

    pub fn toggle_selection(&mut self, display_index: usize) {
        if display_index >= self.filtered_files.len() {
            return;
        }
        let file_index = self.filtered_files[display_index];
        
        let was_selected = self.files[file_index].is_selected;
        self.files[file_index].is_selected = !was_selected;
        
        if !was_selected {
            if !self.selected.contains(&file_index) {
                self.selected.push(file_index);
            }
            self.files[file_index].quantity = 1;
        } else {
            self.selected.retain(|&i| i != file_index);
            self.files[file_index].quantity = 0;
        }
    }

    pub fn select_all(&mut self) {
        self.deselect_all();
        for &file_index in &self.filtered_files {
            self.files[file_index].is_selected = true;
            self.files[file_index].quantity = 1;
            self.selected.push(file_index);
        }
    }

    pub fn deselect_all(&mut self) {
        for file in &mut self.files {
            file.is_selected = false;
            file.quantity = 0;
        }
        self.selected.clear();
        self.editing_index = None;
        self.input_buffer.clear();
    }

    pub fn move_cursor(&mut self, delta: isize, max_visible: usize) {
        let count = self.filtered_files.len();
        if count == 0 {
            self.cursor = 0;
            return;
        }

        let new_pos = self.cursor as isize + delta;
        if new_pos < 0 {
            self.cursor = 0;
        } else if new_pos as usize >= count {
            self.cursor = count - 1;
        } else {
            self.cursor = new_pos as usize;
        }
        
        if self.cursor < self.scroll_offset {
            self.scroll_offset = self.cursor;
        } else if self.cursor >= self.scroll_offset + max_visible {
            self.scroll_offset = self.cursor - max_visible + 1;
        }
    }

    pub fn selected_count(&self) -> usize {
        self.selected.len()
    }

    pub fn total_quantity(&self) -> usize {
        self.files.iter().filter(|f| f.is_selected).map(|f| f.quantity).sum()
    }

    pub fn get_selected_with_quantity(&self) -> Vec<(PathBuf, usize)> {
        self.selected
            .iter()
            .filter_map(|&i| self.files.get(i).map(|f| (f.path.clone(), f.quantity)))
            .collect()
    }

    pub fn start_edit_quantity(&mut self, display_index: usize) {
        if display_index >= self.filtered_files.len() {
            return;
        }
        let file_index = self.filtered_files[display_index];
        if self.files[file_index].is_selected {
            self.editing_index = Some(file_index);
            self.input_buffer = self.files[file_index].quantity.to_string();
        }
    }

    pub fn stop_edit_quantity(&mut self) {
        if let Some(idx) = self.editing_index {
            if idx < self.files.len() {
                if let Ok(qty) = self.input_buffer.parse::<usize>() {
                    self.files[idx].quantity = qty.max(1);
                }
            }
        }
        self.editing_index = None;
        self.input_buffer.clear();
    }

    pub fn cancel_edit_quantity(&mut self) {
        self.editing_index = None;
        self.input_buffer.clear();
    }

    pub fn handle_digit(&mut self, digit: char) {
        if self.editing_index.is_some() {
            self.input_buffer.push(digit);
        } else if self.is_searching {
            self.input_buffer.push(digit);
            self.update_search();
        }
    }

    pub fn handle_backspace(&mut self) {
        if self.editing_index.is_some() {
            self.input_buffer.pop();
        } else if self.is_searching {
            self.input_buffer.pop();
            self.update_search();
        }
    }

    pub fn get_config_from_state(&self) -> Option<crate::config::Config> {
        let edit = self.config_edit_state.as_ref()?;
        Some(crate::config::Config {
            barcode_directory: std::path::PathBuf::from(&edit.barcode_directory),
            printer_name: edit.printer_name.clone(),
            page_size: crate::config::PageSize::Custom,
            temp_directory: std::path::PathBuf::from(&edit.temp_directory),
            preview_command: edit.preview_command.clone(),
            print_command: edit.print_command.clone(),
            max_recent_files: 10,
        })
    }

    pub fn start_config_edit_from_snapshot(snapshot: ConfigSnapshot) -> Self {
        let mut state = Self::default();
        state.mode = AppMode::ConfigEdit;
        state.current_config = Some(snapshot.clone());
        state.config_edit_state = Some(ConfigEditState {
            barcode_directory: snapshot.barcode_directory.to_string_lossy().to_string(),
            printer_name: snapshot.printer_name.clone(),
            temp_directory: snapshot.temp_directory.to_string_lossy().to_string(),
            preview_command: snapshot.preview_command.clone(),
            print_command: snapshot.print_command.clone(),
            selected_field: 0,
            is_editing: false,
            input_buffer: String::new(),
        });
        state
    }

    pub fn toggle_config_edit_mode(&mut self) {
        if let Some(ref mut edit) = self.config_edit_state {
            edit.is_editing = !edit.is_editing;
            if edit.is_editing {
                edit.input_buffer = match edit.selected_field {
                    0 => edit.barcode_directory.clone(),
                    1 => edit.printer_name.clone(),
                    2 => edit.temp_directory.clone(),
                    3 => edit.preview_command.clone(),
                    4 => edit.print_command.clone(),
                    _ => String::new(),
                };
            } else {
                edit.input_buffer.clear();
            }
        }
    }

    pub fn apply_config_edit(&mut self) {
        if let Some(ref mut edit) = self.config_edit_state {
            match edit.selected_field {
                0 => edit.barcode_directory = edit.input_buffer.clone(),
                1 => edit.printer_name = edit.input_buffer.clone(),
                2 => edit.temp_directory = edit.input_buffer.clone(),
                3 => edit.preview_command = edit.input_buffer.clone(),
                4 => edit.print_command = edit.input_buffer.clone(),
                _ => {}
            }
            edit.is_editing = false;
            edit.input_buffer.clear();
        }
    }

    pub fn cancel_config_edit(&mut self) {
        if let Some(ref mut edit) = self.config_edit_state {
            edit.is_editing = false;
            edit.input_buffer.clear();
        }
    }

    pub fn move_config_field(&mut self, delta: isize) {
        if let Some(ref mut edit) = self.config_edit_state {
            if edit.is_editing {
                return;
            }
            let max_fields = 5;
            let new_idx = edit.selected_field as isize + delta;
            if new_idx < 0 {
                edit.selected_field = max_fields - 1;
            } else if new_idx >= max_fields as isize {
                edit.selected_field = 0;
            } else {
                edit.selected_field = new_idx as usize;
            }
        }
    }

    pub fn handle_config_digit(&mut self, digit: char) {
        if let Some(ref mut edit) = self.config_edit_state {
            if edit.is_editing {
                edit.input_buffer.push(digit);
            }
        }
    }

    pub fn handle_config_backspace(&mut self) {
        if let Some(ref mut edit) = self.config_edit_state {
            if edit.is_editing {
                edit.input_buffer.pop();
            }
        }
    }
}