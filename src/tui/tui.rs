use std::io;
use std::path::PathBuf;
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Gauge, List, ListItem, ListState, Paragraph},
    Terminal,
};

#[derive(Debug, Clone, PartialEq)]
pub enum Mode {
    Encode,
    Decode,
}

#[derive(Debug, Clone, PartialEq)]
enum Screen {
    ModeSelect,
    BrowseInput,
    BrowseOutput,
    Processing,
    Result,
}

#[derive(Debug, Clone, PartialEq)]
enum InputMode {
    Navigate,
    ManualPath,
}

struct FileBrowser {
    current_dir: PathBuf,
    all_entries: Vec<PathBuf>,
    filtered_entries: Vec<PathBuf>,
    state: ListState,
    output_name: String,
    search_query: String,
    manual_path: String,
    input_mode: InputMode,
}

impl FileBrowser {
    fn new(start: PathBuf) -> Self {
        let mut browser = FileBrowser {
            current_dir: start,
            all_entries: vec![],
            filtered_entries: vec![],
            state: ListState::default(),
            output_name: String::new(),
            search_query: String::new(),
            manual_path: String::new(),
            input_mode: InputMode::Navigate,
        };
        browser.refresh();
        browser
    }

    fn refresh(&mut self) {
        self.all_entries.clear();

        if self.current_dir.parent().is_some() {
            self.all_entries.push(self.current_dir.join("../../.."));
        }

        if let Ok(rd) = std::fs::read_dir(&self.current_dir) {
            let mut dirs: Vec<PathBuf> = vec![];
            let mut files: Vec<PathBuf> = vec![];

            for entry in rd.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    dirs.push(path);
                } else {
                    files.push(path);
                }
            }

            dirs.sort();
            files.sort();
            self.all_entries.extend(dirs);
            self.all_entries.extend(files);
        }

        self.search_query.clear();
        self.apply_filter();
    }

    fn apply_filter(&mut self) {
        let q = self.search_query.to_lowercase();
        self.filtered_entries = self.all_entries.iter().filter(|p| {
            if q.is_empty() { return true; }
            let name = p.file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_lowercase();
            p.ends_with("../../..") || name.contains(&q)
        }).cloned().collect();

        if !self.filtered_entries.is_empty() {
            self.state.select(Some(0));
        } else {
            self.state.select(None);
        }
    }

    fn entries(&self) -> &Vec<PathBuf> {
        &self.filtered_entries
    }

    fn selected(&self) -> Option<&PathBuf> {
        self.state.selected().and_then(|i| self.filtered_entries.get(i))
    }

    fn enter(&mut self) -> Option<PathBuf> {
        if let Some(path) = self.selected().cloned() {
            if path.ends_with("../../..") {
                if let Some(parent) = self.current_dir.parent() {
                    self.current_dir = parent.to_path_buf();
                    self.refresh();
                }
                return None;
            }
            if path.is_dir() {
                self.current_dir = path;
                self.refresh();
                None
            } else {
                Some(path)
            }
        } else {
            None
        }
    }

    fn jump_to_manual_path(&mut self) -> bool {
        let path = PathBuf::from(&self.manual_path);
        if path.is_dir() {
            self.current_dir = path;
            self.manual_path.clear();
            self.input_mode = InputMode::Navigate;
            self.refresh();
            true
        } else {
            false
        }
    }

    fn up(&mut self) {
        let i = self.state.selected().unwrap_or(0);
        self.state.select(Some(
            if i == 0 { self.filtered_entries.len().saturating_sub(1) } else { i - 1 }
        ));
    }

    fn down(&mut self) {
        let len = self.filtered_entries.len();
        let i = self.state.selected().unwrap_or(0);
        self.state.select(Some(if i + 1 >= len { 0 } else { i + 1 }));
    }
}

pub struct App {
    screen: Screen,
    mode: Mode,
    mode_list_state: ListState,
    input_path: String,
    output_path: String,
    browser: Option<FileBrowser>,
    result_message: String,
    result_ok: bool,
    progress: u16,
    original_size: usize,
    result_size: usize,
}

impl App {
    pub fn new() -> Self {
        let mut state = ListState::default();
        state.select(Some(0));
        App {
            screen: Screen::ModeSelect,
            mode: Mode::Encode,
            mode_list_state: state,
            input_path: String::new(),
            output_path: String::new(),
            browser: None,
            result_message: String::new(),
            result_ok: false,
            progress: 0,
            original_size: 0,
            result_size: 0,
        }
    }
}

pub fn run_tui() -> io::Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new();

    loop {
        terminal.draw(|f| ui(f, &mut app))?;

        if let Event::Key(key) = event::read()? {
            if key.kind != KeyEventKind::Press {
                continue;
            }

            match app.screen {
                Screen::ModeSelect => match key.code {
                    KeyCode::Up => {
                        let i = app.mode_list_state.selected().unwrap_or(0);
                        app.mode_list_state.select(Some(if i == 0 { 1 } else { 0 }));
                    }
                    KeyCode::Down => {
                        let i = app.mode_list_state.selected().unwrap_or(0);
                        app.mode_list_state.select(Some(if i == 1 { 0 } else { 1 }));
                    }
                    KeyCode::Enter => {
                        let i = app.mode_list_state.selected().unwrap_or(0);
                        app.mode = if i == 0 { Mode::Encode } else { Mode::Decode };
                        let start = std::env::current_dir().unwrap_or(PathBuf::from("C:\\"));
                        app.browser = Some(FileBrowser::new(start));
                        app.screen = Screen::BrowseInput;
                    }
                    KeyCode::Char('q') => break,
                    _ => {}
                },

                Screen::BrowseInput => {
                    if let Some(browser) = &mut app.browser {
                        match browser.input_mode {
                            InputMode::ManualPath => match key.code {
                                KeyCode::Enter => { browser.jump_to_manual_path(); }
                                KeyCode::Backspace => { browser.manual_path.pop(); }
                                KeyCode::Char(c) => { browser.manual_path.push(c); }
                                KeyCode::Tab | KeyCode::Esc => {
                                    browser.manual_path.clear();
                                    browser.input_mode = InputMode::Navigate;
                                }
                                _ => {}
                            },
                            InputMode::Navigate => match key.code {
                                KeyCode::Up => browser.up(),
                                KeyCode::Down => browser.down(),
                                KeyCode::Enter => {
                                    if let Some(path) = browser.enter() {
                                        app.input_path = path.to_string_lossy().to_string();
                                        let start = std::env::current_dir()
                                            .unwrap_or(PathBuf::from("C:\\"));
                                        let mut out_browser = FileBrowser::new(start);
                                        let suggested = match app.mode {
                                            Mode::Encode => format!(
                                                "{}.lz77",
                                                path.file_stem().unwrap_or_default().to_string_lossy()
                                            ),
                                            Mode::Decode => format!(
                                                "{}_decoded",
                                                path.file_stem().unwrap_or_default().to_string_lossy()
                                            ),
                                        };
                                        out_browser.output_name = suggested;
                                        app.browser = Some(out_browser);
                                        app.screen = Screen::BrowseOutput;
                                    }
                                }
                                KeyCode::Tab => {
                                    browser.manual_path = browser.current_dir
                                        .to_string_lossy().to_string();
                                    browser.input_mode = InputMode::ManualPath;
                                }
                                KeyCode::Backspace => {
                                    if !browser.search_query.is_empty() {
                                        browser.search_query.pop();
                                        browser.apply_filter();
                                    }
                                }
                                KeyCode::Char(c) => {
                                    browser.search_query.push(c);
                                    browser.apply_filter();
                                }
                                KeyCode::Esc => {
                                    if !browser.search_query.is_empty() {
                                        browser.search_query.clear();
                                        browser.apply_filter();
                                    } else {
                                        app.browser = None;
                                        app.screen = Screen::ModeSelect;
                                    }
                                }
                                _ => {}
                            },
                        }
                    }
                }

                Screen::BrowseOutput => {
                    if let Some(browser) = &mut app.browser {
                        match key.code {
                            KeyCode::Up => browser.up(),
                            KeyCode::Down => browser.down(),
                            KeyCode::Enter => {
                                let out = browser.current_dir.join(&browser.output_name);
                                app.output_path = out.to_string_lossy().to_string();
                                app.browser = None;
                                app.screen = Screen::Processing;
                                app.progress = 50;

                                let result = process(&app.mode, &app.input_path, &app.output_path);
                                match result {
                                    Ok((orig, out_size)) => {
                                        app.original_size = orig;
                                        app.result_size = out_size;
                                        app.result_ok = true;
                                        app.result_message = "Успешно завершено".to_string();
                                    }
                                    Err(e) => {
                                        app.result_ok = false;
                                        app.result_message = format!("Ошибка: {}", e);
                                    }
                                }
                                app.progress = 100;
                                app.screen = Screen::Result;
                            }
                            KeyCode::Backspace => { browser.output_name.pop(); }
                            KeyCode::Char(c) => { browser.output_name.push(c); }
                            KeyCode::Esc => {
                                let start = std::env::current_dir()
                                    .unwrap_or(PathBuf::from("C:\\"));
                                app.browser = Some(FileBrowser::new(start));
                                app.screen = Screen::BrowseInput;
                            }
                            _ => {}
                        }
                    }
                }

                Screen::Processing => {}

                Screen::Result => match key.code {
                    KeyCode::Char('r') => { app = App::new(); }
                    KeyCode::Char('q') => break,
                    _ => {}
                },
            }
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    Ok(())
}

fn process(mode: &Mode, input: &str, output: &str) -> io::Result<(usize, usize)> {
    use crate::encoder::Encoder;
    use crate::decoder::Decoder;
    use crate::serializer::Serializer;
    use std::fs::File;
    use std::io::{BufReader, BufWriter, Read};

    let ser = Serializer::new();
    const CHUNK_SIZE: usize = 4 * 1024 * 1024; // 4 МБ на чанк

    match mode {
        Mode::Encode => {
            let original_size = std::fs::metadata(input)?.len() as usize;
            let enc = Encoder::new(8192, 64);

            let in_file = File::open(input)?;
            let mut reader = BufReader::new(in_file);

            let out_file = File::create(output)?;
            let mut writer = BufWriter::new(out_file);

            ser.write_header(&mut writer)?;

            let mut buf = vec![0u8; CHUNK_SIZE];
            loop {
                let n = reader.read(&mut buf)?;
                if n == 0 { break; }
                let tokens = enc.encode(&buf[..n]);
                for token in &tokens {
                    ser.write_token(&mut writer, token)?;
                }
            }

            ser.write_end(&mut writer)?;
            let result_size = std::fs::metadata(output)?.len() as usize;
            Ok((original_size, result_size))
        }

        Mode::Decode => {
            let original_size = std::fs::metadata(input)?.len() as usize;
            let in_file = File::open(input)?;
            let mut reader = BufReader::new(in_file);
            let tokens = ser.read_tokens(&mut reader)?;
            let dec = Decoder::new();
            let data = dec.decode(&tokens);
            let result_size = data.len();
            std::fs::write(output, &data)?;
            Ok((original_size, result_size))
        }
    }
}

fn ui(f: &mut ratatui::Frame, app: &mut App) {
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
        Screen::Processing => "Обработка...",
        Screen::Result => "r начать заново   q выход",
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
                    let name = if p.ends_with("../../..") {
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