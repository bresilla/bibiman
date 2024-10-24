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

use ratatui::crossterm::event::{
    Event, KeyCode, KeyEvent, KeyModifiers, MouseEvent, MouseEventKind,
};
use tui_input::Input;

// Possible scroll areas.
#[derive(Debug, PartialEq, Eq)]
pub enum ScrollType {
    Rows,
    Cols,
    InfoArea,
}

// Possible ressources to open
#[derive(Debug, PartialEq, Eq)]
pub enum OpenRessource {
    PDF,
    WebLink,
    Note,
}

/// Application command.
#[derive(Debug, PartialEq, Eq)]
pub enum Command {
    // Toggle area
    ToggleArea,
    // Next
    Next(ScrollType, usize),
    // Previous.
    Previous(ScrollType, usize),
    // Go to top.
    Top,
    // Go to bottom.
    Bottom,
    // Search list
    SearchList,
    // Reset lists
    ResetList,
    // Confirm search/selection
    Confirm,
    // Sort table/list
    SortList,
    // Yank selected item
    YankItem,
    // Edit file
    EditFile,
    // Open linked ressource
    Open(OpenRessource),
    // Input command.
    Input(InputCommand),
    // Hexdump command.
    Exit,
    // Do nothing.
    Nothing,
}

impl From<KeyEvent> for Command {
    fn from(key_event: KeyEvent) -> Self {
        match key_event.code {
            // Go to first/last entry of selected list/table
            KeyCode::Char('g') | KeyCode::Home => Self::Top,
            KeyCode::Char('G') | KeyCode::End => Self::Bottom,
            // Scroll columns of EntryTable
            KeyCode::Right | KeyCode::Char('l') => Self::Next(ScrollType::Cols, 1),
            KeyCode::Left | KeyCode::Char('h') => Self::Previous(ScrollType::Cols, 1),
            // Scroll table/list vertically by 1
            KeyCode::Down | KeyCode::Char('j') => Self::Next(ScrollType::Rows, 1),
            KeyCode::Up | KeyCode::Char('k') => Self::Previous(ScrollType::Rows, 1),
            // Scroll table/list vertically by 5
            KeyCode::PageDown => Self::Next(ScrollType::Rows, 5),
            KeyCode::PageUp => Self::Previous(ScrollType::Rows, 5),
            KeyCode::Char('d') => {
                if key_event.modifiers == KeyModifiers::CONTROL {
                    Self::Next(ScrollType::Rows, 5)
                } else {
                    Self::Nothing
                }
            }
            KeyCode::Char('u') => {
                if key_event.modifiers == KeyModifiers::CONTROL {
                    Self::Previous(ScrollType::Rows, 5)
                } else {
                    Self::Open(OpenRessource::WebLink)
                }
            }
            // Exit App
            KeyCode::Char('q') => Self::Exit,
            KeyCode::Char('c') | KeyCode::Char('C') => {
                if key_event.modifiers == KeyModifiers::CONTROL {
                    Self::Exit
                } else {
                    Self::Nothing
                }
            }
            // Switch selected area
            KeyCode::Tab => Self::ToggleArea,
            KeyCode::BackTab => Self::ToggleArea,
            // Enter search mode
            KeyCode::Char('/') => Self::Input(InputCommand::Enter),
            KeyCode::Char('f') => {
                if key_event.modifiers == KeyModifiers::CONTROL {
                    Self::Input(InputCommand::Enter)
                } else {
                    Self::Nothing
                }
            }
            // KeyCode::Backspace => Self::Input(InputCommand::Resume(Event::Key(key_event))),
            // Confirm selection
            KeyCode::Enter => Self::Confirm,
            // Reset lists/tables
            KeyCode::Esc => Self::ResetList,
            // Open linked ressource
            KeyCode::Char('o') => Self::Open(OpenRessource::PDF),
            // KeyCode::Char('u') => Self::Open(OpenRessource::WebLink),
            // Edit currently selected entry
            KeyCode::Char('e') => Self::EditFile,
            // Yank selected item/value
            KeyCode::Char('y') => Self::YankItem,
            // Else do nothing
            _ => Self::Nothing,
        }
    }
}

impl From<MouseEvent> for Command {
    fn from(mouse_event: MouseEvent) -> Self {
        match mouse_event.kind {
            MouseEventKind::ScrollDown => Self::Next(ScrollType::Rows, 1),
            MouseEventKind::ScrollUp => Self::Previous(ScrollType::Rows, 1),
            _ => Self::Nothing,
        }
    }
}

/// Input mode command.
#[derive(Debug, PartialEq, Eq)]
pub enum InputCommand {
    // Handle input.
    Handle(Event),
    // Enter input mode.
    Enter,
    // Confirm input.
    Confirm,
    // Exit input mode
    Exit,
}

impl InputCommand {
    /// Parses the event.
    pub fn parse(key_event: KeyEvent, input: &Input) -> Self {
        if key_event.code == KeyCode::Esc
            || (key_event.code == KeyCode::Backspace && input.value().is_empty())
        {
            Self::Exit
        } else if key_event.code == KeyCode::Enter {
            Self::Confirm
        } else {
            Self::Handle(Event::Key(key_event))
        }
    }
}
