// bibiman - a TUI for managing BibLaTeX databases
// Copyright (C) 2024  lukeflo
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.
/////

use crate::frontend::app::{App, AppResult};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use super::app::{CurrentArea, FormerArea};

/// Handles the key events and updates the state of [`App`].
pub fn handle_key_events(key_event: KeyEvent, app: &mut App) -> AppResult<()> {
    // Keycodes activated for every area (high priority)
    match key_event.code {
        // Exit application on `ESC` or `q`
        KeyCode::Char('Q') | KeyCode::Char('q') => {
            app.quit();
        }
        // Exit application on `Ctrl-C`
        KeyCode::Char('c') | KeyCode::Char('C') => {
            if key_event.modifiers == KeyModifiers::CONTROL {
                app.quit();
            }
        }
        KeyCode::PageDown => {
            app.scroll_info_down();
        }
        KeyCode::PageUp => {
            app.scroll_info_up();
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
            KeyCode::Char('/') => {
                app.former_area = Some(FormerArea::TagArea);
                app.current_area = CurrentArea::SearchArea;
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
            KeyCode::Char('y') => {
                App::yank_text(&app.get_selected_citekey());
            }
            KeyCode::Char('/') => {
                app.former_area = Some(FormerArea::EntryArea);
                app.current_area = CurrentArea::SearchArea;
            }
            KeyCode::Tab | KeyCode::BackTab => {
                app.toggle_area();
            }
            _ => {}
        },
        // Keycodes for the search area (popup)
        CurrentArea::SearchArea => match key_event.code {
            KeyCode::Esc => {
                app.toggle_area();
                app.former_area = None;
                app.search_string.clear();
            }
            KeyCode::Enter => {
                // TODO: run function for filtering the list
                app.toggle_area();
                app.former_area = None;
                app.search_string.clear();
            }
            KeyCode::Backspace => {
                app.search_string.pop();
            }
            KeyCode::Char(search_pattern) => {
                app.search_string.push(search_pattern);
            }
            _ => {}
        },
        // Keycodes for the help area (popup)
        CurrentArea::HelpArea => match key_event.code {
            KeyCode::Char('q') => {
                app.quit();
            }
            KeyCode::Esc => {
                app.toggle_area();
                app.former_area = None;
            }
            _ => {}
        },
    }
    Ok(())
}
