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

use crate::{
    backend::search::BibiSearch,
    frontend::app::{App, AppResult},
};
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
                // app.former_area = Some(FormerArea::TagArea);
                // app.current_area = CurrentArea::SearchArea;
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
                // app.reset_taglist();
            }
            KeyCode::Enter => {
                app.filter_for_tags();
                // app.toggle_area();
                // app.reset_taglist();
                // app.former_area = Some(FormerArea::TagArea);
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
                // if let Some(FormerArea::TagArea) = app.former_area {
                //     app.search_struct.inner_search = true;
                // }
                // app.former_area = Some(FormerArea::EntryArea);
                // app.current_area = CurrentArea::SearchArea;
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
                // app.reset_entry_table();
                // if app.search_struct.inner_search {
                //     app.reset_taglist();
                // }
                // app.former_area = None;
                app.reset_current_list();
            }
            _ => {}
        },
        // Keycodes for the search area (rendered in footer)
        CurrentArea::SearchArea => match key_event.code {
            KeyCode::Esc => {
                // app.toggle_area();
                // if let Some(FormerArea::EntryArea) = app.former_area {
                //     app.reset_entry_table();
                // } else if let Some(FormerArea::TagArea) = app.former_area {
                //     app.reset_taglist();
                // }
                // app.former_area = None;
                // // If search is canceled, reset default status of struct
                // BibiSearch::default();
                app.break_search();
            }
            KeyCode::Enter => {
                // TODO: run function for filtering the list
                // app.toggle_area();
                // app.former_area = Some(FormerArea::SearchArea);
                // // app.search_string.clear();
                // app.search_struct.search_string.clear();
                app.confirm_search();
            }
            KeyCode::Backspace => {
                app.search_struct.search_string.pop();
                if let Some(FormerArea::EntryArea) = app.former_area {
                    app.search_entries();
                } else if let Some(FormerArea::TagArea) = app.former_area {
                    app.search_tags();
                }
            }
            KeyCode::Char(search_pattern) => {
                app.search_struct.search_string.push(search_pattern);
                if let Some(FormerArea::EntryArea) = app.former_area {
                    app.search_entries();
                } else if let Some(FormerArea::TagArea) = app.former_area {
                    app.search_tags();
                }
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
