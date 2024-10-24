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

use crate::bibiman::{Bibiman, CurrentArea};
use crate::tui::Tui;
use crate::App;
use color_eyre::eyre::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

/// Handles the key events and updates the state of [`App`].
pub fn handle_key_events(key_event: KeyEvent, app: &mut App, tui: &mut Tui) -> Result<()> {
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
            app.bibiman.scroll_info_down();
        }
        KeyCode::PageUp => {
            app.bibiman.scroll_info_up();
        }
        _ => {}
    }
    // Keycodes for specific areas
    match app.bibiman.current_area {
        // Keycodes for the tag area
        CurrentArea::TagArea => match key_event.code {
            KeyCode::Down => {
                app.bibiman.select_next_tag(1);
            }
            KeyCode::Up => {
                app.bibiman.select_previous_tag(1);
            }
            KeyCode::Char('j') => {
                if key_event.modifiers == KeyModifiers::ALT {
                    app.bibiman.scroll_info_down();
                } else {
                    app.bibiman.select_next_tag(1);
                }
            }
            KeyCode::Char('k') => {
                if key_event.modifiers == KeyModifiers::ALT {
                    app.bibiman.scroll_info_up();
                } else {
                    app.bibiman.select_previous_tag(1);
                }
            }
            KeyCode::Char('d') => {
                if key_event.modifiers == KeyModifiers::CONTROL {
                    app.bibiman.select_next_tag(5)
                }
            }
            KeyCode::Char('u') => {
                if key_event.modifiers == KeyModifiers::CONTROL {
                    app.bibiman.select_previous_tag(5)
                }
            }
            KeyCode::Char('g') | KeyCode::Home => {
                app.bibiman.select_first_tag();
            }
            KeyCode::Char('G') | KeyCode::End => {
                app.bibiman.select_last_tag();
            }
            KeyCode::Char('/') => {
                app.bibiman.enter_search_area();
            }
            KeyCode::Char('f') | KeyCode::Char('F') => {
                if key_event.modifiers == KeyModifiers::CONTROL {
                    app.bibiman.enter_search_area();
                }
            }
            KeyCode::Tab | KeyCode::BackTab => {
                app.bibiman.toggle_area();
            }
            KeyCode::Esc => {
                app.bibiman.reset_current_list();
            }
            KeyCode::Enter => {
                app.bibiman.filter_for_tags();
            }
            _ => {}
        },
        // Keycodes for the entry area
        CurrentArea::EntryArea => match key_event.code {
            KeyCode::Down => {
                app.bibiman.select_next_entry(1);
            }
            KeyCode::Up => {
                app.bibiman.select_previous_entry(1);
            }
            KeyCode::Char('j') => {
                if key_event.modifiers == KeyModifiers::ALT {
                    app.bibiman.scroll_info_down();
                } else {
                    app.bibiman.select_next_entry(1);
                }
            }
            KeyCode::Char('k') => {
                if key_event.modifiers == KeyModifiers::ALT {
                    app.bibiman.scroll_info_up();
                } else {
                    app.bibiman.select_previous_entry(1);
                }
            }
            KeyCode::Char('d') => {
                if key_event.modifiers == KeyModifiers::CONTROL {
                    app.bibiman.select_next_entry(5);
                }
            }
            KeyCode::Char('u') => {
                if key_event.modifiers == KeyModifiers::CONTROL {
                    app.bibiman.select_previous_entry(5);
                } else {
                    app.bibiman.open_doi_url()?;
                }
            }
            KeyCode::Char('g') | KeyCode::Home => {
                app.bibiman.select_first_entry();
            }
            KeyCode::Char('G') | KeyCode::End => {
                app.bibiman.select_last_entry();
            }
            KeyCode::Char('h') => {
                app.bibiman.select_prev_column();
            }
            KeyCode::Char('l') => {
                app.bibiman.select_next_column();
            }
            KeyCode::Char('s') => {
                app.bibiman.entry_table.sort_entry_table(true);
            }
            KeyCode::Char('y') => {
                Bibiman::yank_text(&app.bibiman.get_selected_citekey());
            }
            KeyCode::Char('e') => {
                app.bibiman.run_editor(tui)?;
            }
            KeyCode::Char('o') => {
                app.bibiman.open_connected_file()?;
            }
            KeyCode::Char('/') => {
                app.bibiman.enter_search_area();
            }
            KeyCode::Char('f') | KeyCode::Char('F') => {
                if key_event.modifiers == KeyModifiers::CONTROL {
                    app.bibiman.enter_search_area();
                }
            }
            KeyCode::Tab | KeyCode::BackTab => {
                app.bibiman.toggle_area();
            }
            KeyCode::Esc => {
                app.bibiman.reset_current_list();
            }
            _ => {}
        },
        // Keycodes for the search area (rendered in footer)
        CurrentArea::SearchArea => match key_event.code {
            KeyCode::Esc => {
                app.bibiman.break_search();
            }
            KeyCode::Enter => {
                app.bibiman.confirm_search();
            }
            KeyCode::Backspace => {
                app.bibiman.search_pattern_pop();
            }
            KeyCode::Char(search_pattern) => {
                app.bibiman.search_pattern_push(search_pattern);
            }
            _ => {}
        },
        // Keycodes for the help area (popup)
        CurrentArea::HelpArea => match key_event.code {
            KeyCode::Char('q') => {
                app.quit();
            }
            KeyCode::Esc => {
                app.bibiman.toggle_area();
                app.bibiman.former_area = None;
            }
            _ => {}
        },
        CurrentArea::InfoArea => {}
    }
    Ok(())
}
