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

use ratatui::{
    style::{Color, Stylize},
    text::{Line, Span, Text},
    widgets::ListState,
};

use crate::cliargs::CLIArgs;

#[derive(Debug)]
pub enum PopupKind {
    Help,
    MessageConfirm,
    MessageError,
    Selection,
    AddEntry,
}

#[derive(Debug, Default)]
pub struct PopupArea {
    pub is_popup: bool,
    pub popup_kind: Option<PopupKind>,
    pub popup_message: String,
    pub popup_scroll_pos: u16,
    pub popup_list: Vec<String>,
    pub popup_state: ListState,
    pub add_entry_input: String,
    pub add_entry_cursor_position: usize,
}

impl PopupArea {
    pub fn popup_help<'a>(args: &CLIArgs) -> Text<'a> {
        let help = [
            ("General", "first"),
            ("TAB: ", "Toggle areas (Entries, Keywords)"),
            ("/|Ctrl+f: ", "Enter search mode"),
            ("q|Ctrl+c: ", "Quit bibiman"),
            ("a: ", "Add new entry"),
            ("?: ", "Show help"),
            ("Entry Table", "sub"),
            ("j,k|↓,↑: ", "Select next/previous entry"),
            ("Ctrl+d,Ctrl+u: ", "Select fifth-next/previous entry"),
            ("h,l|←,→: ", "Select next/previous column"),
            ("g|Home: ", "Go to first entry"),
            ("G|End: ", "Go to last entry"),
            ("s: ", "sort entries by selected column (toggles reversed)"),
            ("y: ", "yank/copy citekey of selected entry to clipboard"),
            ("e: ", "Open editor at selected entry"),
            ("o: ", "Open with selected entry associated PDF"),
            ("u: ", "Open DOI/URL of selected entry"),
            ("ESC: ", "Reset all lists"),
            ("Keyword List", "sub"),
            ("j,k|↓,↑: ", "Select next/previous item"),
            ("Ctrl+d,Ctrl+u: ", "Select fifth-next/previous entry"),
            ("g|Home: ", "Go to first keyword"),
            ("G|End: ", "Go to last keyword"),
            ("ENTER: ", "Filter by selected keyword"),
            ("Search", "sub"),
            ("↓,↑,←,→: ", "Move cursor"),
            ("BACKSPACE: ", "Delete Character"),
            ("ENTER: ", "Confirm search"),
            ("ESC: ", "Abort search"),
        ];

        let mut helptext: Vec<Line<'_>> = vec![];

        for (keys, help) in help {
            if help == "first" {
                helptext.push(Line::from(
                    Span::raw(keys)
                        .bold()
                        .fg(Color::Indexed(args.colors.main_text_color)),
                ))
            } else if help == "sub" {
                helptext.push(Line::from(""));
                helptext.push(Line::from(
                    Span::raw(keys)
                        .bold()
                        .fg(Color::Indexed(args.colors.main_text_color)),
                ))
            } else {
                helptext.push(Line::from(vec![
                    Span::raw(keys)
                        .bold()
                        .fg(Color::Indexed(args.colors.keyword_color)),
                    Span::raw(help),
                ]))
            }
        }

        Text::from(helptext)
    }

    pub fn popup_message(&mut self, message: &str, object: &str, msg_confirm: bool) {
        if object.is_empty() {
            self.popup_message = message.to_owned();
        } else {
            self.popup_message = message.to_owned() + object; //format!("{} \"{}\"", message, object);
        }
        if msg_confirm {
            self.popup_kind = Some(PopupKind::MessageConfirm);
        } else {
            self.popup_kind = Some(PopupKind::MessageError)
        }
        self.is_popup = true;
    }

    pub fn popup_selection(&mut self, items: Vec<String>) {
        self.popup_list = items;
        self.popup_kind = Some(PopupKind::Selection);
        self.is_popup = true;
    }

    pub fn popup_scroll_down(&mut self) {
        self.popup_scroll_pos = self.popup_scroll_pos.saturating_add(1)
    }

    pub fn popup_scroll_up(&mut self) {
        self.popup_scroll_pos = self.popup_scroll_pos.saturating_sub(1)
    }
}
