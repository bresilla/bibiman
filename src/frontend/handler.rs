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

use crate::frontend::app::App;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use super::app::{CurrentArea, FormerArea};
use color_eyre::eyre::Result;

/// Handles the key events and updates the state of [`App`].
pub fn handle_key_events(key_event: KeyEvent, app: &mut App) -> Result<()> {
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
                app.select_next_tag();
            }
            KeyCode::Char('k') | KeyCode::Up => {
                app.select_previous_tag();
            }
            KeyCode::Char('g') | KeyCode::Home => {
                app.select_first_tag();
            }
            KeyCode::Char('G') | KeyCode::End => {
                app.select_last_tag();
            }
            KeyCode::Char('/') => {
                app.enter_search_area();
            }
            KeyCode::Char('f') | KeyCode::Char('F') => {
                if key_event.modifiers == KeyModifiers::CONTROL {
                    app.enter_search_area();
                }
            }
            KeyCode::Tab | KeyCode::BackTab => {
                app.toggle_area();
            }
            KeyCode::Esc => {
                app.reset_current_list();
            }
            KeyCode::Enter => {
                app.filter_for_tags();
            }
            _ => {}
        },
        // Keycodes for the entry area
        CurrentArea::EntryArea => match key_event.code {
            KeyCode::Char('j') | KeyCode::Down => {
                app.select_next_entry();
            }
            KeyCode::Char('k') | KeyCode::Up => {
                app.select_previous_entry();
            }
            KeyCode::Char('g') | KeyCode::Home => {
                app.select_first_entry();
            }
            KeyCode::Char('G') | KeyCode::End => {
                app.select_last_entry();
            }
            KeyCode::Char('y') => {
                App::yank_text(&app.get_selected_citekey());
            }
            KeyCode::Char('/') => {
                app.enter_search_area();
            }
            KeyCode::Char('f') | KeyCode::Char('F') => {
                if key_event.modifiers == KeyModifiers::CONTROL {
                    app.enter_search_area();
                }
            }
            KeyCode::Tab | KeyCode::BackTab => {
                app.toggle_area();
            }
            KeyCode::Esc => {
                app.reset_current_list();
            }
            _ => {}
        },
        // Keycodes for the search area (rendered in footer)
        CurrentArea::SearchArea => match key_event.code {
            KeyCode::Esc => {
                app.break_search();
            }
            KeyCode::Enter => {
                app.confirm_search();
            }
            KeyCode::Backspace => {
                app.search_pattern_pop();
            }
            KeyCode::Char(search_pattern) => {
                app.search_pattern_push(search_pattern);
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
