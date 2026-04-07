use std::path::PathBuf;
use ratatui::widgets::ListState;
use super::app::InputMode;

pub struct FileBrowser {
    pub current_dir: PathBuf,
    pub all_entries: Vec<PathBuf>,
    pub filtered_entries: Vec<PathBuf>,
    pub state: ListState,
    pub output_name: String,
    pub search_query: String,
    pub manual_path: String,
    pub input_mode: InputMode,
}

impl FileBrowser {
    pub fn new(start: PathBuf) -> Self {
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

    pub fn refresh(&mut self) {
        self.all_entries.clear();

        if self.current_dir.parent().is_some() {
            self.all_entries.push(self.current_dir.join(".."));
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

    pub fn apply_filter(&mut self) {
        let q = self.search_query.to_lowercase();
        self.filtered_entries = self.all_entries.iter().filter(|p| {
            if q.is_empty() { return true; }
            let name = p.file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_lowercase();
            p.ends_with("..") || name.contains(&q)
        }).cloned().collect();

        if !self.filtered_entries.is_empty() {
            self.state.select(Some(0));
        } else {
            self.state.select(None);
        }
    }

    pub fn entries(&self) -> &Vec<PathBuf> {
        &self.filtered_entries
    }

    pub fn selected(&self) -> Option<&PathBuf> {
        self.state.selected().and_then(|i| self.filtered_entries.get(i))
    }

    pub fn enter(&mut self) -> Option<PathBuf> {
        if let Some(path) = self.selected().cloned() {
            if path.ends_with("..") {
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

    pub fn jump_to_manual_path(&mut self) -> bool {
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

    pub fn up(&mut self) {
        let i = self.state.selected().unwrap_or(0);
        self.state.select(Some(
            if i == 0 { self.filtered_entries.len().saturating_sub(1) } else { i - 1 }
        ));
    }

    pub fn down(&mut self) {
        let len = self.filtered_entries.len();
        let i = self.state.selected().unwrap_or(0);
        self.state.select(Some(if i + 1 >= len { 0 } else { i + 1 }));
    }
}