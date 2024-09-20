use crate::frontend::app::{App, AppResult};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use super::app::CurrentArea;

/// Handles the key events and updates the state of [`App`].
pub fn handle_key_events(key_event: KeyEvent, app: &mut App) -> AppResult<()> {
    // Keycodes activated for every area (high priority)
    match key_event.code {
        // Exit application on `ESC` or `q`
        KeyCode::Esc | KeyCode::Char('q') => {
            app.quit();
        }
        // Exit application on `Ctrl-C`
        KeyCode::Char('c') | KeyCode::Char('C') => {
            if key_event.modifiers == KeyModifiers::CONTROL {
                app.quit();
            }
        }
        _ => {}
    }
    // Keycodes for specific areas
    match app.current_area {
        // Keycodes for the tag area
        CurrentArea::TagArea => match key_event.code {
            KeyCode::Char('j') | KeyCode::Down => {
                app.select_next();
            }
            KeyCode::Char('k') | KeyCode::Up => {
                app.select_previous();
            }
            KeyCode::Char('h') | KeyCode::Left => {
                app.select_none();
            }
            KeyCode::Char('g') | KeyCode::Home => {
                app.select_first();
            }
            KeyCode::Char('G') | KeyCode::End => {
                app.select_last();
            }
            KeyCode::Tab | KeyCode::BackTab => {
                app.toggle_area();
            }
            _ => {}
        },
        // Keycodes for the entry area
        CurrentArea::EntryArea => match key_event.code {
            KeyCode::Char('j') | KeyCode::Down => {
                app.select_next();
            }
            KeyCode::Char('k') | KeyCode::Up => {
                app.select_previous();
            }
            KeyCode::Char('h') | KeyCode::Left => {
                app.select_none();
            }
            KeyCode::Char('g') | KeyCode::Home => {
                app.select_first();
            }
            KeyCode::Char('G') | KeyCode::End => {
                app.select_last();
            }
            KeyCode::Tab | KeyCode::BackTab => {
                app.toggle_area();
            }
            _ => {}
        },
    }
    Ok(())
}
