use ratatui::{
    Frame,
    widgets::*,
    layout::*,
    style::*,
    text::{Line, Span},
};
use ratatui::crossterm::event::KeyCode;
use ratatui::crossterm::event::KeyEventKind;
use crate::tui::{AppState, AppMode, ConfigSnapshot};

pub const COLOR_SELECTED: Color = Color::Green;
pub const COLOR_CURSOR: Color = Color::Blue;
pub const COLOR_TITLE: Color = Color::Cyan;
pub const COLOR_INFO: Color = Color::Yellow;

pub fn render_ui(frame: &mut Frame, state: &AppState) {
    match state.mode {
        AppMode::Select => render_select_mode(frame, state),
        AppMode::ConfirmPrint => render_confirm_mode(frame, state),
        AppMode::Printing => render_printing_mode(frame, state),
        AppMode::Success => render_success_mode(frame, state),
        AppMode::Error => render_error_mode(frame, state),
        AppMode::ConfigEdit => render_config_edit_mode(frame, state),
    }
}

fn render_select_mode(frame: &mut Frame, state: &AppState) {
    let area = frame.area();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(if state.is_searching { 5 } else { 3 }),
            Constraint::Min(0),
            Constraint::Length(3),
            Constraint::Length(4),
        ])
        .split(area);

    let title = Paragraph::new(Line::from(vec![
        ratatui::text::Span::raw(" Ozon Print - Выбор штрихкодов "),
        ratatui::text::Span::styled(
            format!("[{} шт.]", state.total_quantity()),
            Style::default().fg(COLOR_SELECTED).bold(),
        ),
    ]))
    .block(
        Block::default()
            .title(" Выберите штрихкоды для печати ")
            .title_style(Style::default().fg(COLOR_TITLE).bold())
            .borders(Borders::ALL)
            .border_style(Style::default().fg(COLOR_TITLE)),
    )
    .style(Style::default().bg(Color::Black));
    frame.render_widget(title, chunks[0]);

    if state.is_searching {
        let search_content = vec![
            Line::from(vec![
                ratatui::text::Span::styled("Поиск: ", Style::default().fg(COLOR_INFO)),
                ratatui::text::Span::styled(
                    if state.input_buffer.is_empty() { "_".to_string() } else { state.input_buffer.clone() },
                    Style::default().fg(COLOR_SELECTED).bold(),
                ),
                ratatui::text::Span::styled(
                    format!(" ({})", state.display_count()),
                    Style::default().fg(Color::DarkGray),
                ),
            ]),
            Line::from(vec![
                ratatui::text::Span::styled("Esc", Style::default().fg(Color::Red)),
                ratatui::text::Span::raw(" - отмена"),
            ]),
        ];

        let search_block = Paragraph::new(search_content)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(COLOR_INFO)),
            )
            .style(Style::default().bg(Color::Black));
        frame.render_widget(search_block, chunks[1]);
    } else if state.files.is_empty() {
        let empty = Paragraph::new("PDF файлы не найдены в указанной директории")
            .style(Style::default().fg(Color::Red))
            .block(Block::default().borders(Borders::ALL).title(" Пусто "));
        frame.render_widget(empty, chunks[1]);
    } else {
        let table_rows: Vec<Row> = state.filtered_files
            .iter()
            .enumerate()
            .map(|(display_idx, &file_idx)| {
                let file = &state.files[file_idx];
                let is_cursor = display_idx == state.cursor;
                let is_editing = state.editing_index == Some(file_idx);
                
                let checkbox = if file.is_selected {
                    "[x]"
                } else {
                    "[ ]"
                };

                let qty_display = if is_editing {
                    state.input_buffer.clone()
                } else if file.is_selected {
                    file.quantity.to_string()
                } else {
                    "-".to_string()
                };

                let base_style = if is_cursor {
                    Style::default()
                        .fg(Color::Black)
                        .bg(COLOR_CURSOR)
                } else if file.is_selected {
                    Style::default()
                        .fg(COLOR_SELECTED)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::White)
                };

                Row::new(vec![
                    Cell::from(Span::styled(checkbox, base_style.clone())),
                    Cell::from(Span::styled(&file.name, base_style.clone())),
                    Cell::from(Span::styled(format!("{} KB", file.size_kb), Style::default().fg(Color::DarkGray))),
                    Cell::from(Span::styled(qty_display, if is_editing {
                        Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD).add_modifier(Modifier::UNDERLINED)
                    } else if file.is_selected {
                        Style::default().fg(COLOR_SELECTED).add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(Color::DarkGray)
                    })),
                ])
            })
            .collect();

        let widths = [
            Constraint::Length(4),
            Constraint::Min(30),
            Constraint::Length(12),
            Constraint::Length(8),
        ];

        let table = Table::new(table_rows, &widths)
            .column_spacing(1)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::DarkGray)),
            )
            .highlight_style(Style::default().add_modifier(Modifier::REVERSED));

        frame.render_widget(table, chunks[1]);
    }

    let info_text = if state.files.is_empty() {
        Line::from("Директория пуста")
    } else if state.is_searching {
        Line::from(vec![
            ratatui::text::Span::raw("Найдено: "),
            ratatui::text::Span::styled(format!("{} / {}", state.display_count(), state.files.len()), Style::default().fg(COLOR_INFO)),
        ])
    } else {
        Line::from(vec![
            ratatui::text::Span::raw("Найдено: "),
            ratatui::text::Span::styled(format!("{}", state.files.len()), Style::default().fg(COLOR_INFO)),
            ratatui::text::Span::raw("  |  "),
            ratatui::text::Span::raw("Выбрано: "),
            ratatui::text::Span::styled(format!("{}", state.selected_count()), Style::default().fg(COLOR_SELECTED)),
            ratatui::text::Span::raw("  |  "),
            ratatui::text::Span::raw("Всего шт.: "),
            ratatui::text::Span::styled(format!("{}", state.total_quantity()), Style::default().fg(COLOR_SELECTED).bold()),
        ])
    };

    let info = Paragraph::new(info_text)
        .style(Style::default().fg(Color::White))
        .block(Block::default().borders(Borders::ALL));
    frame.render_widget(info, chunks[2]);

    let help_lines = vec![
        Line::from(vec![
            ratatui::text::Span::raw("["),
            ratatui::text::Span::styled("↑↓", Style::default().fg(COLOR_CURSOR)),
            ratatui::text::Span::raw("] Навигация  "),
            ratatui::text::Span::raw("["),
            ratatui::text::Span::styled("Space", Style::default().fg(COLOR_CURSOR)),
            ratatui::text::Span::raw("] Вкл/Выкл  "),
            ratatui::text::Span::raw("["),
            ratatui::text::Span::styled("e", Style::default().fg(COLOR_CURSOR)),
            ratatui::text::Span::raw("] Кол-во  "),
            ratatui::text::Span::raw("["),
            ratatui::text::Span::styled("i", Style::default().fg(COLOR_INFO)),
            ratatui::text::Span::raw("] Поиск  "),
            ratatui::text::Span::raw("["),
            ratatui::text::Span::styled("a", Style::default().fg(COLOR_CURSOR)),
            ratatui::text::Span::raw("] Все  "),
            ratatui::text::Span::raw("["),
            ratatui::text::Span::styled("n", Style::default().fg(COLOR_CURSOR)),
            ratatui::text::Span::raw("] Снять  "),
            ratatui::text::Span::raw("["),
            ratatui::text::Span::styled("c", Style::default().fg(COLOR_INFO)),
            ratatui::text::Span::raw("] Настройки  "),
            ratatui::text::Span::raw("["),
            ratatui::text::Span::styled("Enter", Style::default().fg(COLOR_SELECTED)),
            ratatui::text::Span::raw("] Печать  "),
            ratatui::text::Span::raw("["),
            ratatui::text::Span::styled("q", Style::default().fg(Color::Red)),
            ratatui::text::Span::raw("] Выход"),
        ])
    ];

    let help = Paragraph::new(help_lines)
        .style(Style::default().fg(Color::DarkGray))
        .block(Block::default());
    frame.render_widget(help, chunks[3]);
}

fn render_confirm_mode(frame: &mut Frame, state: &AppState) {
    let area = frame.area();

    let title_text = Paragraph::new(Line::from(vec![
        ratatui::text::Span::styled(" Подтверждение печати ", Style::default().fg(COLOR_TITLE).bold()),
    ]));

    let selected_items = state.get_selected_with_quantity();
    
    let content = vec![
        Line::from(""),
        Line::from(vec![
            ratatui::text::Span::raw("Всего позиций: "),
            ratatui::text::Span::styled(format!("{}", selected_items.len()), Style::default().fg(COLOR_SELECTED).bold()),
            ratatui::text::Span::raw("  |  "),
            ratatui::text::Span::raw("Всего штук: "),
            ratatui::text::Span::styled(format!("{}", state.total_quantity()), Style::default().fg(COLOR_SELECTED).bold()),
        ]),
        Line::from(""),
        Line::from("Выбранные штрихкоды:"),
    ];

    let file_lines: Vec<Line> = selected_items
        .iter()
        .map(|(path, qty)| {
            let name = path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown");
            Line::from(vec![
                ratatui::text::Span::styled("• ", Style::default().fg(COLOR_SELECTED)),
                ratatui::text::Span::raw(name.to_string()),
                ratatui::text::Span::raw("  "),
                ratatui::text::Span::styled(format!("x{}", qty), Style::default().fg(COLOR_INFO)),
            ])
        })
        .collect();

    let actions = vec![
        Line::from(""),
        Line::from(vec![
            ratatui::text::Span::raw("["),
            ratatui::text::Span::styled("Enter", Style::default().fg(COLOR_SELECTED)),
            ratatui::text::Span::raw("] Начать печать   "),
            ratatui::text::Span::raw("["),
            ratatui::text::Span::styled("Esc", Style::default().fg(Color::Red)),
            ratatui::text::Span::raw("] Назад к выбору"),
        ]),
    ];

    let all_lines: Vec<Line> = content
        .into_iter()
        .chain(file_lines)
        .chain(actions)
        .collect();

    let paragraph = Paragraph::new(all_lines)
        .block(
            Block::default()
                .title(" Подтверждение ")
                .title_style(Style::default().fg(COLOR_TITLE).bold())
                .borders(Borders::ALL)
                .border_style(Style::default().fg(COLOR_TITLE)),
        )
        .style(Style::default().bg(Color::Black));

    frame.render_widget(title_text, area);
    frame.render_widget(paragraph, area);
}

fn render_printing_mode(frame: &mut Frame, _state: &AppState) {
    let area = frame.area();

    let spinner_text = Line::from(vec![
        ratatui::text::Span::styled("⟳", Style::default().fg(COLOR_TITLE)),
        ratatui::text::Span::raw("  Отправка на печать..."),
    ]);

    let content = Paragraph::new(spinner_text)
        .block(
            Block::default()
                .title(" Печать ")
                .title_style(Style::default().fg(COLOR_TITLE).bold())
                .borders(Borders::ALL)
                .border_style(Style::default().fg(COLOR_TITLE)),
        )
        .style(Style::default().bg(Color::Black))
        .alignment(Alignment::Center);

    frame.render_widget(content, area);
}

fn render_success_mode(frame: &mut Frame, _state: &AppState) {
    let area = frame.area();

    let content = Paragraph::new(vec![
        Line::from(""),
        Line::from(vec![
            ratatui::text::Span::styled("✓ ", Style::default().fg(COLOR_SELECTED).bold()),
            ratatui::text::Span::styled("Печать успешно запущена!", Style::default().fg(COLOR_SELECTED).bold()),
        ]),
        Line::from(""),
        Line::from("Документ отправлен на принтер."),
        Line::from(""),
        Line::from(vec![
            ratatui::text::Span::raw("["),
            ratatui::text::Span::styled("Enter", Style::default().fg(COLOR_CURSOR)),
            ratatui::text::Span::raw("] Начать заново   "),
            ratatui::text::Span::raw("["),
            ratatui::text::Span::styled("q", Style::default().fg(Color::Red)),
            ratatui::text::Span::raw("] Выход"),
        ]),
    ])
    .block(
        Block::default()
            .title(" Успех ")
            .title_style(Style::default().fg(COLOR_SELECTED).bold())
            .borders(Borders::ALL)
            .border_style(Style::default().fg(COLOR_SELECTED)),
    )
    .style(Style::default().bg(Color::Black))
    .alignment(Alignment::Center);

    frame.render_widget(content, area);
}

fn render_error_mode(frame: &mut Frame, _state: &AppState) {
    let area = frame.area();

    let content = Paragraph::new(vec![
        Line::from(""),
        Line::from(vec![
            ratatui::text::Span::styled("✗ ", Style::default().fg(Color::Red).bold()),
            ratatui::text::Span::styled("Ошибка печати", Style::default().fg(Color::Red).bold()),
        ]),
        Line::from(""),
        Line::from("Проверьте настройки принтера."),
        Line::from(""),
        Line::from(vec![
            ratatui::text::Span::raw("["),
            ratatui::text::Span::styled("Enter", Style::default().fg(COLOR_CURSOR)),
            ratatui::text::Span::raw("] Назад   "),
            ratatui::text::Span::raw("["),
            ratatui::text::Span::styled("q", Style::default().fg(Color::Red)),
            ratatui::text::Span::raw("] Выход"),
        ]),
    ])
    .block(
        Block::default()
            .title(" Ошибка ")
            .title_style(Style::default().fg(Color::Red).bold())
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Red)),
    )
    .style(Style::default().bg(Color::Black))
    .alignment(Alignment::Center);

    frame.render_widget(content, area);
}

pub fn handle_key_event(key: &ratatui::crossterm::event::KeyEvent, state: &mut AppState) -> Option<TuiAction> {
    if key.kind != KeyEventKind::Press {
        return None;
    }

    match state.mode {
        AppMode::Select => handle_select_mode(key, state),
        AppMode::ConfirmPrint => handle_confirm_mode(key, state),
        AppMode::Printing => handle_printing_mode(key, state),
        AppMode::Success => handle_success_mode(key, state),
        AppMode::Error => handle_error_mode(key, state),
        AppMode::ConfigEdit => handle_config_edit_mode(key, state),
    }
}

fn handle_select_mode(key: &ratatui::crossterm::event::KeyEvent, state: &mut AppState) -> Option<TuiAction> {
    if state.editing_index.is_some() {
        return handle_edit_quantity_mode(key, state);
    }

    if state.is_searching {
        return handle_search_mode(key, state);
    }

    let max_visible = 20;

    match key.code {
        KeyCode::Up | KeyCode::Char('k') => {
            state.move_cursor(-1, max_visible);
            None
        }
        KeyCode::Down | KeyCode::Char('j') => {
            state.move_cursor(1, max_visible);
            None
        }
        KeyCode::Char(' ') => {
            state.toggle_selection(state.cursor);
            None
        }
        KeyCode::Char('a') => {
            state.select_all();
            None
        }
        KeyCode::Char('n') => {
            state.deselect_all();
            None
        }
        KeyCode::Char('e') => {
            state.start_edit_quantity(state.cursor);
            None
        }
        KeyCode::Char('i') => {
            state.start_search();
            None
        }
        KeyCode::Char('c') => {
            if let Some(ref config) = state.current_config {
                let snapshot = ConfigSnapshot {
                    barcode_directory: config.barcode_directory.clone(),
                    printer_name: config.printer_name.clone(),
                    temp_directory: config.temp_directory.clone(),
                    preview_command: config.preview_command.clone(),
                    print_command: config.print_command.clone(),
                };
                *state = AppState::start_config_edit_from_snapshot(snapshot);
            }
            None
        }
        KeyCode::Enter => {
            if state.selected_count() > 0 {
                state.mode = AppMode::ConfirmPrint;
            }
            None
        }
        KeyCode::Char('q') => {
            Some(TuiAction::Quit)
        }
        _ => None,
    }
}

fn handle_search_mode(key: &ratatui::crossterm::event::KeyEvent, state: &mut AppState) -> Option<TuiAction> {
    let max_visible = 20;

    match key.code {
        KeyCode::Char(c) if !c.is_control() => {
            state.input_buffer.push(c);
            state.update_search();
            None
        }
        KeyCode::Backspace => {
            state.input_buffer.pop();
            state.update_search();
            None
        }
        KeyCode::Up | KeyCode::Char('k') => {
            state.move_cursor(-1, max_visible);
            None
        }
        KeyCode::Down | KeyCode::Char('j') => {
            state.move_cursor(1, max_visible);
            None
        }
        KeyCode::Char(' ') => {
            if state.display_count() > 0 {
                state.toggle_selection(state.cursor);
            }
            None
        }
        KeyCode::Esc => {
            state.stop_search();
            None
        }
        KeyCode::Enter => {
            if state.selected_count() > 0 {
                state.stop_search();
                state.mode = AppMode::ConfirmPrint;
            } else if state.display_count() > 0 && state.is_searching {
                state.toggle_selection(state.cursor);
            }
            None
        }
        _ => None,
    }
}

fn handle_edit_quantity_mode(key: &ratatui::crossterm::event::KeyEvent, state: &mut AppState) -> Option<TuiAction> {
    match key.code {
        KeyCode::Char(c) if c.is_ascii_digit() => {
            state.handle_digit(c);
            None
        }
        KeyCode::Backspace => {
            state.handle_backspace();
            None
        }
        KeyCode::Enter => {
            state.stop_edit_quantity();
            None
        }
        KeyCode::Esc => {
            state.cancel_edit_quantity();
            None
        }
        _ => None,
    }
}

fn handle_confirm_mode(key: &ratatui::crossterm::event::KeyEvent, state: &mut AppState) -> Option<TuiAction> {
    match key.code {
        KeyCode::Enter => {
            state.mode = AppMode::Printing;
            Some(TuiAction::StartPrint(state.get_selected_with_quantity()))
        }
        KeyCode::Esc => {
            state.mode = AppMode::Select;
            None
        }
        _ => None,
    }
}

fn handle_printing_mode(_key: &ratatui::crossterm::event::KeyEvent, _state: &mut AppState) -> Option<TuiAction> {
    None
}

fn handle_success_mode(key: &ratatui::crossterm::event::KeyEvent, state: &mut AppState) -> Option<TuiAction> {
    match key.code {
        KeyCode::Enter => {
            state.mode = AppMode::Select;
            state.deselect_all();
            None
        }
        KeyCode::Char('q') | KeyCode::Esc => {
            Some(TuiAction::Quit)
        }
        _ => None,
    }
}

fn handle_error_mode(key: &ratatui::crossterm::event::KeyEvent, state: &mut AppState) -> Option<TuiAction> {
    match key.code {
        KeyCode::Enter => {
            state.mode = AppMode::Select;
            None
        }
        KeyCode::Char('q') | KeyCode::Esc => {
            Some(TuiAction::Quit)
        }
        _ => None,
    }
}

#[derive(Debug, Clone)]
pub enum TuiAction {
    Quit,
    StartPrint(Vec<(std::path::PathBuf, usize)>),
    SaveConfig,
}

fn render_config_edit_mode(frame: &mut Frame, state: &AppState) {
    let area = frame.area();

    let title = Paragraph::new(Line::from(vec![
        ratatui::text::Span::styled(" Настройки ", Style::default().fg(COLOR_TITLE).bold()),
    ]))
    .block(
        Block::default()
            .title(" Редактирование config_for_ozon_print.json ")
            .title_style(Style::default().fg(COLOR_TITLE).bold())
            .borders(Borders::ALL)
            .border_style(Style::default().fg(COLOR_TITLE)),
    )
    .style(Style::default().bg(Color::Black));

    frame.render_widget(title, area);

    let inner = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(4),
        ])
        .split(area);

    let fields = if let Some(ref edit) = state.config_edit_state {
        vec![
            ("Директория штрихкодов:", &edit.barcode_directory, 0),
            ("Принтер:", &edit.printer_name, 1),
            ("Временная папка:", &edit.temp_directory, 2),
            ("Команда предпросмотра:", &edit.preview_command, 3),
            ("Команда печати:", &edit.print_command, 4),
        ]
    } else {
        return;
    };

    for (i, (label, value, field_idx)) in fields.iter().enumerate() {
        let is_selected = state.config_edit_state.as_ref().map(|e| e.selected_field == *field_idx).unwrap_or(false);
        let is_editing = state.config_edit_state.as_ref().map(|e| e.is_editing).unwrap_or(false);
        
        let display_value = if is_editing && is_selected {
            if state.config_edit_state.as_ref().map(|e| e.input_buffer.is_empty()).unwrap_or(false) { 
                "_".to_string() 
            } else { 
                state.config_edit_state.as_ref().map(|e| e.input_buffer.clone()).unwrap_or_default() 
            }
        } else if value.is_empty() { 
            "_".to_string() 
        } else { 
            value.to_string() 
        };
        
        let value_style = if is_editing && is_selected {
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD).add_modifier(Modifier::UNDERLINED)
        } else if is_selected {
            Style::default().fg(COLOR_SELECTED).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::White)
        };
        
        let line = Line::from(vec![
            ratatui::text::Span::raw(format!("{:25}", format!("{} ", label))),
            ratatui::text::Span::styled(display_value, value_style),
        ]);
        
        let block = if is_selected {
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(if is_editing { Color::Yellow } else { COLOR_SELECTED }))
        } else {
            Block::default().borders(Borders::NONE)
        };
        
        let para = Paragraph::new(line).block(block).style(Style::default().bg(Color::Black));
        frame.render_widget(para, inner[i]);
    }

    let help_lines = vec![
        Line::from(vec![
            ratatui::text::Span::raw("["),
            ratatui::text::Span::styled("↑↓", Style::default().fg(COLOR_CURSOR)),
            ratatui::text::Span::raw("] Навигация  "),
            ratatui::text::Span::raw("["),
            ratatui::text::Span::styled("j/k", Style::default().fg(COLOR_CURSOR)),
            ratatui::text::Span::raw("] Навигация  "),
            ratatui::text::Span::raw("["),
            ratatui::text::Span::styled("Space", Style::default().fg(COLOR_CURSOR)),
            ratatui::text::Span::raw("] Вкл/Выкл  "),
        ]),
        Line::from(vec![
            ratatui::text::Span::raw("["),
            ratatui::text::Span::styled("Enter", Style::default().fg(COLOR_SELECTED)),
            ratatui::text::Span::raw("] Сохранить  "),
            ratatui::text::Span::raw("["),
            ratatui::text::Span::styled("Esc", Style::default().fg(Color::Red)),
            ratatui::text::Span::raw("] Отмена"),
        ]),
    ];

    let help = Paragraph::new(help_lines)
        .block(Block::default().borders(Borders::ALL))
        .style(Style::default().bg(Color::Black).fg(Color::DarkGray));
    frame.render_widget(help, inner[6]);
}

fn handle_config_edit_mode(key: &ratatui::crossterm::event::KeyEvent, state: &mut AppState) -> Option<TuiAction> {
    let is_editing = state.config_edit_state.as_ref().map(|e| e.is_editing).unwrap_or(false);
    
    if is_editing {
        match key.code {
            KeyCode::Char(c) if !c.is_control() => {
                state.handle_config_digit(c);
                None
            }
            KeyCode::Backspace => {
                state.handle_config_backspace();
                None
            }
            KeyCode::Enter => {
                state.apply_config_edit();
                None
            }
            KeyCode::Esc => {
                state.cancel_config_edit();
                None
            }
            _ => None,
        }
    } else {
        match key.code {
            KeyCode::Up | KeyCode::Char('k') => {
                state.move_config_field(-1);
                None
            }
            KeyCode::Down | KeyCode::Char('j') => {
                state.move_config_field(1);
                None
            }
            KeyCode::Char(' ') => {
                state.toggle_config_edit_mode();
                None
            }
            KeyCode::Enter => {
                return Some(TuiAction::SaveConfig);
            }
            KeyCode::Esc | KeyCode::Char('q') => {
                state.mode = AppMode::Select;
                state.config_edit_state = None;
                None
            }
            _ => None,
        }
    }
}