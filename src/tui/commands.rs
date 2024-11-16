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

// // Possible ressources to open
// #[derive(Debug, PartialEq, Eq)]
// pub enum OpenRessource {
//     Pdf,
//     WebLink,
//     Note,
// }

/// Application command.
#[derive(Debug, PartialEq, Eq)]
pub enum CmdAction {
    // Toggle area
    ToggleArea,
    // Scroll list/table down
    SelectNextRow(u16),
    // Scroll list/table up.
    SelectPrevRow(u16),
    // Select nex table col.
    SelectNextCol,
    // Select previous table col.
    SelectPrevCol,
    // Scroll info/preview area down
    ScrollInfoDown,
    // Scroll info/preview area up
    ScrollInfoUp,
    // Go to top.
    Top,
    // Go to bottom.
    Bottom,
    // Search list
    SearchList,
    // Reset lists
    Reset,
    // Confirm search/selection
    Confirm,
    // Sort table/list
    SortList,
    // Yank selected item
    YankItem,
    // Edit file
    EditFile,
    // Open linked ressource
    Open,
    // Input command.
    Input(InputCmdAction),
    // Hexdump command.
    Exit,
    // Show keybindings
    ShowHelp,
    // Do nothing.
    Nothing,
}

impl From<KeyEvent> for CmdAction {
    fn from(key_event: KeyEvent) -> Self {
        match key_event.code {
            // Go to first/last entry of selected list/table
            KeyCode::Char('g') | KeyCode::Home => Self::Top,
            KeyCode::Char('G') | KeyCode::End => Self::Bottom,
            // Scroll columns of EntryTable
            KeyCode::Right | KeyCode::Char('l') => Self::SelectNextCol,
            KeyCode::Left | KeyCode::Char('h') => Self::SelectPrevCol,
            // Scroll table/list vertically by 1
            KeyCode::Down | KeyCode::Char('j') => {
                if key_event.modifiers == KeyModifiers::ALT {
                    Self::ScrollInfoDown
                } else {
                    Self::SelectNextRow(1)
                }
            }
            KeyCode::Up | KeyCode::Char('k') => {
                if key_event.modifiers == KeyModifiers::ALT {
                    Self::ScrollInfoUp
                } else {
                    Self::SelectPrevRow(1)
                }
            }
            // Scroll table/list vertically by 5
            KeyCode::Char('d') => {
                if key_event.modifiers == KeyModifiers::CONTROL {
                    Self::SelectNextRow(5)
                } else {
                    Self::Nothing
                }
            }
            KeyCode::Char('u') => {
                if key_event.modifiers == KeyModifiers::CONTROL {
                    Self::SelectPrevRow(5)
                } else {
                    Self::Nothing
                    // Self::Open(OpenRessource::WebLink)
                }
            }
            // Scroll info/preview area
            KeyCode::PageDown => Self::ScrollInfoDown,
            KeyCode::PageUp => Self::ScrollInfoUp,
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
            KeyCode::Char('/') => Self::Input(InputCmdAction::Enter),
            KeyCode::Char('f') => {
                if key_event.modifiers == KeyModifiers::CONTROL {
                    Self::Input(InputCmdAction::Enter)
                } else {
                    Self::Nothing
                }
            }
            // KeyCode::Backspace => Self::Input(InputCommand::Resume(Event::Key(key_event))),
            // Confirm selection
            KeyCode::Enter => Self::Confirm,
            // Reset lists/tables
            KeyCode::Esc => Self::Reset,
            // Open linked ressource
            KeyCode::Char('o') => Self::Open,
            // KeyCode::Char('u') => Self::Open(OpenRessource::WebLink),
            // Edit currently selected entry
            KeyCode::Char('e') => Self::EditFile,
            // Yank selected item/value
            KeyCode::Char('y') => Self::YankItem,
            // Sort entry table by selected col
            KeyCode::Char('s') => Self::SortList,
            // Show help popup
            KeyCode::Char('?') => Self::ShowHelp,
            // Else do nothing
            _ => Self::Nothing,
        }
    }
}

impl From<MouseEvent> for CmdAction {
    fn from(mouse_event: MouseEvent) -> Self {
        match mouse_event.kind {
            MouseEventKind::ScrollDown => Self::SelectNextRow(1),
            MouseEventKind::ScrollUp => Self::SelectPrevRow(1),
            _ => Self::Nothing,
        }
    }
}

/// Input mode command.
#[derive(Debug, PartialEq, Eq)]
pub enum InputCmdAction {
    // Handle input.
    Handle(Event),
    // Enter input mode.
    Enter,
    // Confirm input.
    Confirm,
    // Exit input mode
    Exit,
    // Do nothing
    Nothing,
}

impl InputCmdAction {
    /// Parses the event.
    pub fn parse(key_event: KeyEvent, input: &Input) -> Self {
        if key_event.code == KeyCode::Backspace && input.value().is_empty() {
            Self::Nothing
        } else if key_event.code == KeyCode::Esc {
            Self::Exit
        } else if key_event.code == KeyCode::Enter {
            Self::Confirm
        } else {
            Self::Handle(Event::Key(key_event))
        }
    }
}
