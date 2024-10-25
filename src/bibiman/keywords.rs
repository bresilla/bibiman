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

use ratatui::widgets::{ListState, ScrollbarState};

#[derive(Debug)]
pub struct TagList {
    pub tag_list_items: Vec<String>,
    pub tag_list_at_search_start: Vec<String>,
    pub tag_list_state: ListState,
    pub tag_scroll_state: ScrollbarState,
    pub selected_keywords: Vec<String>,
}

// Structure of the list items.
#[derive(Debug)]
pub struct TagListItem {
    pub keyword: String,
}

// Function to process inputed characters and convert them (to string, or more complex function)
impl TagListItem {
    pub fn new(info: &str) -> Self {
        Self {
            keyword: info.to_string(),
        }
    }
}

impl TagList {
    pub fn new(keyword_list: Vec<String>) -> Self {
        let tag_list_items = keyword_list;
        let tag_list_state = ListState::default(); // for preselection: .with_selected(Some(0));
        let tag_scroll_state = ScrollbarState::new(tag_list_items.len());
        Self {
            tag_list_items,
            tag_list_at_search_start: Vec::new(),
            tag_list_state,
            tag_scroll_state,
            selected_keywords: Vec::new(),
        }
    }
}
