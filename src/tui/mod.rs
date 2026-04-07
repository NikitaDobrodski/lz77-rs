pub mod app;
pub mod browser;
pub mod process;
pub mod ui;

use std::io;
use std::path::PathBuf;
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use app::{App, Screen, InputMode, Mode};
use browser::FileBrowser;
use process::process;
use ui::ui;

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