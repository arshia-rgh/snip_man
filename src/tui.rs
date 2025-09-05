//! Terminal user interface (TUI) for interactive snippet search and copy.
//!
//! Key bindings:
//! - Type to filter by description (fuzzy)
//! - Up/Down to navigate
//! - Enter to copy selected snippet to clipboard and exit
//! - q to quit without copying

use crate::Snippet;
use arboard::Clipboard;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use fuzzy_matcher::FuzzyMatcher;
use fuzzy_matcher::skim::SkimMatcherV2;
use ratatui::{
    Frame, Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
};
use std::io;

/// In-memory state for the interactive app.
struct App {
    all_snippets: Vec<Snippet>,
    visible_snippets: Vec<Snippet>,
    list_state: ListState,
    search_query: String,
    matcher: SkimMatcherV2,
}

impl App {
    fn new(snippets: Vec<Snippet>) -> App {
        let visible_snippets = snippets.clone();
        App {
            all_snippets: snippets,
            visible_snippets,
            list_state: ListState::default(),
            search_query: String::new(),
            matcher: SkimMatcherV2::default(),
        }
    }

    fn filter_snippets(&mut self) {
        if self.search_query.is_empty() {
            self.visible_snippets = self.all_snippets.clone();
        } else {
            let query = self.search_query.clone();
            let matcher = &self.matcher;

            let mut scored: Vec<(Snippet, i64)> = self
                .all_snippets
                .iter()
                .filter_map(|snippet| {
                    let mut best: Option<i64> = None;

                    if let Some(s) = matcher.fuzzy_match(&snippet.description, &query) {
                        best = Some(s);
                    }
                    if let Some(s) = matcher.fuzzy_match(&snippet.tags.join(" "), &query) {
                        best = Some(best.map_or(s, |b| b.max(s)));
                    }
                    if let Some(s) = matcher.fuzzy_match(&snippet.code, &query) {
                        best = Some(best.map_or(s, |b| b.max(s)));
                    }

                    best.map(|score| (snippet.clone(), score))
                })
                .collect();

            scored.sort_by(|a, b| b.1.cmp(&a.1));

            self.visible_snippets = scored.into_iter().map(|(snip, _)| snip).collect();
        }
        if !self.visible_snippets.is_empty() {
            self.list_state.select(Some(0));
        } else {
            self.list_state.select(None);
        }
    }

    pub fn next(&mut self) {
        let i = match self.list_state.selected() {
            Some(i) => {
                if self.visible_snippets.is_empty() {
                    0
                } else if i >= self.visible_snippets.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.list_state.select(Some(i));
    }

    pub fn previous(&mut self) {
        let i = match self.list_state.selected() {
            Some(i) => {
                if self.visible_snippets.is_empty() {
                    0
                } else if i == 0 {
                    self.visible_snippets.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.list_state.select(Some(i));
    }
}

/// Run the TUI and return the selected snippet's code if Enter is pressed.
/// Returns Ok(None) if the user quits without selecting.
pub fn run_tui(all_snippets: Vec<Snippet>) -> io::Result<Option<String>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new(all_snippets);
    app.list_state.select(Some(0));

    let mut selected_code: Option<String> = None;

    loop {
        terminal.draw(|f| ui(f, &mut app))?;

        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char('q') => {
                    break;
                }
                KeyCode::Enter => {
                    if let Some(selected_index) = app.list_state.selected() {
                        if let Some(selected_snippet) = app.visible_snippets.get(selected_index) {
                            selected_code = Some(selected_snippet.code.clone());
                            break;
                        }
                    }
                }
                KeyCode::Down => app.next(),
                KeyCode::Up => app.previous(),
                KeyCode::Char(c) => {
                    app.search_query.push(c);
                    app.filter_snippets();
                }
                KeyCode::Backspace => {
                    app.search_query.pop();
                    app.filter_snippets();
                }
                _ => {}
            }
        }
    }

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Some(code_to_copy) = selected_code {
        let mut clipboard = Clipboard::new().expect("Failed to initialize clipboard");
        clipboard
            .set_text(code_to_copy.clone())
            .expect("Failed to copy text to clipboard");
        return Ok(Some(code_to_copy));
    }

    Ok(None)
}

fn ui(f: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([Constraint::Length(3), Constraint::Min(0)].as_ref())
        .split(f.area());

    let search_bar = Paragraph::new(app.search_query.as_str())
        .block(Block::default().borders(Borders::ALL).title("Search"));
    f.render_widget(search_bar, chunks[0]);

    let items: Vec<ListItem> = app
        .visible_snippets
        .iter()
        .map(|s| ListItem::new(s.description.as_str()))
        .collect();

    let snippets_list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("Snippets"))
        .highlight_style(
            Style::default()
                .bg(Color::Rgb(0, 150, 150))
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">> ");

    f.render_stateful_widget(snippets_list, chunks[1], &mut app.list_state);
}
