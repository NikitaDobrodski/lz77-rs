use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Direction, Layout};
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, Borders, Gauge, List, ListItem, Paragraph};
use super::app::{App, Screen, InputMode};

pub fn ui(f: &mut Frame, app: &mut App) {
    let area = f.area();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(3),
        ])
        .split(area);

    let title = Paragraph::new("LZ77 Compressor")
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(title, chunks[0]);

    let hint = match app.screen {
        Screen::ModeSelect => "↑↓ выбор   Enter подтвердить   q выход",
        Screen::BrowseInput => {
            if let Some(b) = &app.browser {
                if b.input_mode == InputMode::ManualPath {
                    "Введите путь к папке   Enter перейти   Esc отмена"
                } else {
                    "↑↓ навигация   Enter выбрать   Tab ввод пути   буквы = поиск   Esc сброс/назад"
                }
            } else { "" }
        }
        Screen::BrowseOutput => "↑↓ выбор папки   Backspace/буквы = имя файла   Enter сохранить   Esc назад",
        Screen::Processing   => "Обработка...",
        Screen::Result       => "r начать заново   q выход",
    };
    let footer = Paragraph::new(hint)
        .style(Style::default().fg(Color::DarkGray))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(footer, chunks[2]);

    match app.screen {
        Screen::ModeSelect => {
            let items = vec![
                ListItem::new("  Сжать файл (encode)"),
                ListItem::new("  Распаковать файл (decode)"),
            ];
            let list = List::new(items)
                .block(Block::default().borders(Borders::ALL).title(" Выберите режим "))
                .highlight_style(
                    Style::default().fg(Color::Black).bg(Color::Cyan).add_modifier(Modifier::BOLD),
                )
                .highlight_symbol("> ");
            f.render_stateful_widget(list, chunks[1], &mut app.mode_list_state);
        }

        Screen::BrowseInput | Screen::BrowseOutput => {
            if let Some(browser) = &mut app.browser {
                let layout = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        Constraint::Length(3),
                        Constraint::Length(3),
                        Constraint::Min(0),
                    ])
                    .split(chunks[1]);

                let dir_str = browser.current_dir.to_string_lossy().to_string();
                let path_widget = Paragraph::new(dir_str)
                    .style(Style::default().fg(Color::Yellow))
                    .block(Block::default().borders(Borders::ALL).title(" Текущая папка "));
                f.render_widget(path_widget, layout[0]);

                if app.screen == Screen::BrowseInput {
                    match &browser.input_mode {
                        InputMode::Navigate => {
                            let search_text = if browser.search_query.is_empty() {
                                "Начните вводить для поиска...".to_string()
                            } else {
                                format!("{}_", browser.search_query)
                            };
                            let search_color = if browser.search_query.is_empty() {
                                Color::DarkGray
                            } else {
                                Color::White
                            };
                            let search = Paragraph::new(search_text)
                                .style(Style::default().fg(search_color))
                                .block(Block::default().borders(Borders::ALL).title(" Поиск "));
                            f.render_widget(search, layout[1]);
                        }
                        InputMode::ManualPath => {
                            let path_text = format!("{}_", browser.manual_path);
                            let manual = Paragraph::new(path_text)
                                .style(Style::default().fg(Color::Green))
                                .block(Block::default().borders(Borders::ALL)
                                    .title(" Перейти по пути (Tab/Esc = отмена) ")
                                    .border_style(Style::default().fg(Color::Green)));
                            f.render_widget(manual, layout[1]);
                        }
                    }
                } else {
                    let name_text = format!("{}_", browser.output_name);
                    let name_field = Paragraph::new(name_text)
                        .style(Style::default().fg(Color::Green))
                        .block(Block::default().borders(Borders::ALL).title(" Имя выходного файла "));
                    f.render_widget(name_field, layout[1]);
                }

                let list_title = if app.screen == Screen::BrowseInput {
                    if !browser.search_query.is_empty() {
                        format!(" Результаты поиска: {} ", browser.filtered_entries.len())
                    } else {
                        " Выберите файл ".to_string()
                    }
                } else {
                    " Выберите папку для сохранения ".to_string()
                };

                let items: Vec<ListItem> = browser.entries().iter().map(|p| {
                    let name = if p.ends_with("..") {
                        "..".to_string()
                    } else if p.is_dir() {
                        format!("[{}]", p.file_name().unwrap_or_default().to_string_lossy())
                    } else {
                        p.file_name().unwrap_or_default().to_string_lossy().to_string()
                    };
                    ListItem::new(name)
                }).collect();

                let list = List::new(items)
                    .block(Block::default().borders(Borders::ALL).title(list_title.as_str()))
                    .highlight_style(
                        Style::default().fg(Color::Black).bg(Color::Cyan).add_modifier(Modifier::BOLD),
                    )
                    .highlight_symbol("> ");

                f.render_stateful_widget(list, layout[2], &mut browser.state);
            }
        }

        Screen::Processing => {
            let gauge = Gauge::default()
                .block(Block::default().borders(Borders::ALL).title(" Обработка "))
                .gauge_style(Style::default().fg(Color::Cyan))
                .percent(app.progress);
            f.render_widget(gauge, chunks[1]);
        }

        Screen::Result => {
            let inner = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(4), Constraint::Min(0)])
                .split(chunks[1]);

            let color = if app.result_ok { Color::Green } else { Color::Red };
            let status = Paragraph::new(app.result_message.clone())
                .block(Block::default().borders(Borders::ALL).title(" Результат "))
                .style(Style::default().fg(color).add_modifier(Modifier::BOLD))
                .alignment(Alignment::Center);
            f.render_widget(status, inner[0]);

            if app.result_ok {
                let ratio = if app.original_size > 0 {
                    app.result_size as f64 / app.original_size as f64 * 100.0
                } else { 0.0 };

                let stats = format!(
                    "Входной файл:   {} байт\nВыходной файл:  {} байт\nСоотношение:    {:.1}%\nСохранено в:    {}",
                    app.original_size,
                    app.result_size,
                    ratio,
                    app.output_path,
                );
                let p = Paragraph::new(stats)
                    .block(Block::default().borders(Borders::ALL).title(" Статистика "))
                    .style(Style::default().fg(Color::White));
                f.render_widget(p, inner[1]);
            }
        }
    }
}