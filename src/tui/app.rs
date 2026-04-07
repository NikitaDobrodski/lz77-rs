use ratatui::widgets::ListState;
use super::browser::FileBrowser;

#[derive(Debug, Clone, PartialEq)]
pub enum Mode {
    Encode,
    Decode,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Screen {
    ModeSelect,
    BrowseInput,
    BrowseOutput,
    Processing,
    Result,
}

#[derive(Debug, Clone, PartialEq)]
pub enum InputMode {
    Navigate,
    ManualPath,
}

pub struct App {
    pub screen: Screen,
    pub mode: Mode,
    pub mode_list_state: ListState,
    pub input_path: String,
    pub output_path: String,
    pub browser: Option<FileBrowser>,
    pub result_message: String,
    pub result_ok: bool,
    pub progress: u16,
    pub original_size: usize,
    pub result_size: usize,
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