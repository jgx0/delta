use std::{io, time::Duration};

use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, List, ListItem, Paragraph},
};

use crate::models::RepoSnapshot;

pub fn run(snapshot: &RepoSnapshot) -> Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let outcome = loop {
        terminal.draw(|f| render(f, snapshot))?;

        if event::poll(Duration::from_millis(150))? {
            if let Event::Key(key) = event::read()? {
                if matches!(key.code, KeyCode::Char('q') | KeyCode::Esc) {
                    break Ok(());
                }
            }
        }
    };

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    outcome
}

fn render(frame: &mut Frame<'_>, snapshot: &RepoSnapshot) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(5), Constraint::Min(8)])
        .split(frame.area());

    let summary = Paragraph::new(format!(
        "commits: {}   contributors: {}   files: {}   +{} / -{}",
        snapshot.stats.commit_count,
        snapshot.contributors.len(),
        snapshot.file_churn.len(),
        snapshot.stats.lines_added,
        snapshot.stats.lines_deleted,
    ))
    .block(Block::default().title("Delta Overview").borders(Borders::ALL));
    frame.render_widget(summary, chunks[0]);

    let bottom = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(chunks[1]);

    let contributors = snapshot
        .contributors
        .iter()
        .take(10)
        .map(|c| ListItem::new(format!("{} ({})", c.name, c.commit_count)))
        .collect::<Vec<_>>();
    let left = List::new(contributors)
        .block(Block::default().title("Top Contributors").borders(Borders::ALL));

    let hotspots = snapshot
        .hotspots
        .iter()
        .take(10)
        .map(|h| ListItem::new(format!("{:.3} {}", h.score, h.path)))
        .collect::<Vec<_>>();
    let right = List::new(hotspots)
        .block(Block::default().title("Hotspots").borders(Borders::ALL));

    frame.render_widget(left, bottom[0]);
    frame.render_widget(right, bottom[1]);
}
