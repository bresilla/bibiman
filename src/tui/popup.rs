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

use crate::MAIN_PURPLE_COLOR_INDEX;

#[derive(Debug)]
pub enum PopupKind {
    Help,
    Message,
    Selection,
}

#[derive(Debug)]
pub struct PopupArea {
    pub is_popup: bool,
    pub popup_kind: Option<PopupKind>,
    pub popup_message: String,
    pub popup_list: Vec<String>,
    pub popup_state: ListState,
}

impl Default for PopupArea {
    fn default() -> Self {
        PopupArea {
            is_popup: false,
            popup_kind: None,
            popup_message: String::new(),
            popup_list: Vec::new(),
            popup_state: ListState::default(),
        }
    }
}

impl PopupArea {
    pub fn popup_help<'a>() -> Text<'a> {
        let help = [
            ("j,k|↓,↑: ", "Select next/previous item"),
            ("h,l|←,→: ", "Select next/previous column (Entry table)"),
            ("g|Home: ", "Go to first item"),
            ("G|End: ", "Go to last item"),
            ("s: ", "sort entries by selected column (toggles reversed)"),
            ("TAB: ", "Toggle areas (Entries, Keywords)"),
            ("/|Ctrl+f: ", "Enter search mode"),
            ("y: ", "yank/copy citekey of selected entry to clipboard"),
            ("e: ", "Open editor at selected entry"),
            ("o: ", "Open with selected entry associated PDF"),
            ("u: ", "Open DOI/URL of selected entry"),
            ("ESC: ", "Reset all lists/abort search"),
            ("ENTER: ", "Confirm search/filter by selected keyword"),
            ("q|Ctrl+c: ", "Quit bibiman"),
        ];

        let help_text: Vec<Line<'_>> = help
            .into_iter()
            .map(|(keys, help)| {
                Line::from(vec![
                    Span::raw(keys)
                        .bold()
                        .fg(Color::Indexed(MAIN_PURPLE_COLOR_INDEX)),
                    Span::raw(help),
                ])
            })
            .collect();

        let text = Text::from(help_text);
        text
    }

    pub fn popup_message(&mut self, message: &str, object: String) {
        if object.is_empty() {
            self.popup_message = message.into();
        } else {
            self.popup_message = format!("{} \"{}\"", message, object);
        }
        self.popup_kind = Some(PopupKind::Message);
        self.is_popup = true;
    }

    pub fn popup_close_message(&mut self) {
        self.is_popup = false;
        self.popup_message.clear();
        self.popup_kind = None
    }
}
