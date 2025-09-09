//! Terminal user interface (TUI) for interactive snippet search and copy.
//!
//! Key bindings:
//! - Type to filter by description (fuzzy)
//! - Up/Down to navigate
//! - Enter to copy selected snippet to clipboard and exit
//! - q to quit without copying
//! - p: preview selected snippet code
//! - d: delete selected snippet
//! - PgUp/PgDn: scroll preview up/down

use crate::snippets::{Snippet, delete_snippet};
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
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap},
};
use std::io;

enum Mode {
    Normal,
    ConfirmDelete,
}

/// In-memory state for the interactive app.
struct App {
    all_snippets: Vec<Snippet>,
    visible_snippets: Vec<usize>,
    list_state: ListState,
    search_query: String,
    matcher: SkimMatcherV2,
    mode: Mode,
    preview_full: bool,
    preview_scroll: u16,
    status_msg: Option<String>,
}

impl App {
    fn new(snippets: Vec<Snippet>) -> App {
        let visible_indices = (0..snippets.len()).collect();
        App {
            all_snippets: snippets,
            visible_snippets: visible_indices,
            list_state: ListState::default(),
            search_query: String::new(),
            matcher: SkimMatcherV2::default(),
            mode: Mode::Normal,
            preview_full: false,
            preview_scroll: 0,
            status_msg: None,
        }
    }

    fn filter_snippets(&mut self) {
        if self.search_query.is_empty() {
            self.visible_snippets = (0..self.all_snippets.len()).collect();
        } else {
            let query = self.search_query.as_str();
            let matcher = &self.matcher;

            let mut scored: Vec<(usize, i64)> = self
                .all_snippets
                .iter()
                .enumerate()
                .filter_map(|(idx, snippet)| {
                    let mut best: Option<i64> = None;

                    if let Some(s) = matcher.fuzzy_match(&snippet.description, query) {
                        best = Some(s);
                    }
                    if let Some(s) = matcher.fuzzy_match(&snippet.tags.join(" "), query) {
                        best = Some(best.map_or(s, |b| b.max(s)));
                    }
                    if let Some(s) = matcher.fuzzy_match(&snippet.code, query) {
                        best = Some(best.map_or(s, |b| b.max(s)));
                    }

                    best.map(|score| (idx, score))
                })
                .collect();

            scored.sort_by(|a, b| b.1.cmp(&a.1));
            self.visible_snippets = scored.into_iter().map(|(idx, _)| idx).collect();
        }

        if !self.visible_snippets.is_empty() {
            self.list_state.select(Some(0));
        } else {
            self.list_state.select(None);
        }
        self.preview_scroll = 0;
    }

    fn next(&mut self) {
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
        self.preview_scroll = 0;
    }

    fn previous(&mut self) {
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
        self.preview_scroll = 0;
    }

    fn selected_snippet(&self) -> Option<&Snippet> {
        self.list_state
            .selected()
            .and_then(|i| self.visible_snippets.get(i))
            .and_then(|&idx| self.all_snippets.get(idx))
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

    let mut selected_code: Option<&str> = None;

    loop {
        terminal.draw(|f| ui(f, &mut app))?;

        if let Event::Key(key) = event::read()? {
            match app.mode {
                Mode::ConfirmDelete => match key.code {
                    KeyCode::Char('y') => {
                        if let Some(sel) = app.list_state.selected() {
                            if let Some(&idx) = app.visible_snippets.get(sel) {
                                let id = app.all_snippets[idx].id.clone();
                                match delete_snippet(&id) {
                                    Ok(_) => {
                                        app.all_snippets.retain(|s| s.id != id);
                                        app.filter_snippets();

                                        if app.visible_snippets.is_empty() {
                                            app.list_state.select(None);
                                        } else {
                                            let new_sel = sel.min(app.visible_snippets.len() - 1);
                                            app.list_state.select(Some(new_sel));
                                        }
                                        app.status_msg = Some("Deleted snippet.".to_string());
                                    }
                                    Err(e) => {
                                        app.status_msg = Some(format!("Delete failed: {}", e));
                                    }
                                }
                            }
                        }
                        app.mode = Mode::Normal;
                    }
                    KeyCode::Char('n') | KeyCode::Esc => {
                        app.mode = Mode::Normal;
                        app.status_msg = Some("Canceled delete.".to_string());
                    }
                    _ => {}
                },
                Mode::Normal => match key.code {
                    KeyCode::Char('q') => break,
                    KeyCode::Enter => {
                        if let Some(selected_index) = app.list_state.selected() {
                            if let Some(&selected_snippet) =
                                app.visible_snippets.get(selected_index)
                            {
                                selected_code =
                                    Some(app.all_snippets[selected_snippet].code.as_str());
                                break;
                            }
                        }
                    }
                    KeyCode::Down => app.next(),
                    KeyCode::Up => app.previous(),
                    KeyCode::PageDown => {
                        let max_lines = app
                            .selected_snippet()
                            .map(|s| s.code.lines().count())
                            .unwrap_or(0);
                        let max_scroll = max_lines.saturating_sub(1) as u16;
                        app.preview_scroll = (app.preview_scroll.saturating_add(5)).min(max_scroll);
                    }
                    KeyCode::PageUp => {
                        app.preview_scroll = app.preview_scroll.saturating_sub(5);
                    }
                    KeyCode::Char('p') => {
                        app.preview_full = !app.preview_full;
                        app.preview_scroll = 0;
                    }
                    KeyCode::Char('d') => {
                        app.mode = Mode::ConfirmDelete;
                        app.status_msg = Some("Confirm delete? press 'y' or 'n'".to_string());
                    }
                    KeyCode::Backspace => {
                        app.search_query.pop();
                        app.filter_snippets();
                    }
                    KeyCode::Char(c) => {
                        app.search_query.push(c);
                        app.filter_snippets();
                    }
                    _ => {}
                },
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
            .set_text(code_to_copy)
            .expect("Failed to copy text to clipboard");
        return Ok(Some(code_to_copy.to_string()));
    }

    Ok(None)
}

fn ui(f: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([Constraint::Length(3), Constraint::Min(0)].as_ref())
        .split(f.area());

    let mut title = "Search".to_string();
    match app.mode {
        Mode::ConfirmDelete => title.push_str(" [confirm delete: y/n]"),
        Mode::Normal => {}
    }
    if let Some(msg) = &app.status_msg {
        title.push_str(" • ");
        title.push_str(msg);
    }
    let search_bar = Paragraph::new(app.search_query.as_str())
        .block(Block::default().borders(Borders::ALL).title(title));
    f.render_widget(search_bar, chunks[0]);

    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(55), Constraint::Percentage(45)].as_ref())
        .split(chunks[1]);

    let items: Vec<ListItem> = app
        .visible_snippets
        .iter()
        .map(|&i| ListItem::new(app.all_snippets[i].description.as_str()))
        .collect();

    let snippets_list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Snippets (Enter copy, d delete, p preview, PgUp/PgDn scroll, q quit)"),
        )
        .highlight_style(
            Style::default()
                .bg(Color::Rgb(0, 150, 150))
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">> ");

    f.render_stateful_widget(snippets_list, main_chunks[0], &mut app.list_state);

    let preview_text = if let Some(s) = app.selected_snippet() {
        if app.preview_full {
            s.code.clone()
        } else {
            let mut lines: Vec<&str> = s.code.lines().collect();
            let truncated = if lines.len() > 10 {
                lines.truncate(10);
                let mut t = lines.join("\n");
                t.push_str("\n…");
                t
            } else {
                lines.join("\n")
            };
            truncated
        }
    } else {
        String::from("No snippet selected.")
    };

    let preview = Paragraph::new(preview_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(if app.preview_full {
                    "Preview (full)"
                } else {
                    "Preview (compact)"
                }),
        )
        .wrap(Wrap { trim: false })
        .scroll((app.preview_scroll, 0));

    f.render_widget(preview, main_chunks[1]);
}
